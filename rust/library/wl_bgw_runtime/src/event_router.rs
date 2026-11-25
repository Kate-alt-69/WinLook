//! Bounded, non-blocking fan-out for normalized Traton events.
//!
//! BGW owns this router and publishes only from its host loop. Each subscriber
//! receives an independent bounded mailbox; a slow or disconnected consumer
//! cannot block or prevent delivery to the other consumers.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};

use wl_traton::{EventQueueError, TratonEvent, TratonEventKind};

pub(crate) const DEFAULT_EVENT_MAILBOX_CAPACITY: usize = 64;
const MAX_EVENT_MAILBOX_CAPACITY: usize = 4096;
const MAX_EVENT_SUBSCRIBERS: usize = 32;
const MAX_SUBSCRIBER_ID_BYTES: usize = 64;

/// A subscriber's explicit interest set. Empty subscriptions are rejected so
/// a forgotten filter cannot silently become a no-op registration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EventKindFilter {
    kinds: BTreeSet<TratonEventKind>,
}

impl EventKindFilter {
    pub(crate) fn only(
        kinds: impl IntoIterator<Item = TratonEventKind>,
    ) -> Result<Self, EventSubscriptionError> {
        let kinds: BTreeSet<_> = kinds.into_iter().collect();
        if kinds.is_empty() {
            return Err(EventSubscriptionError::EmptyFilter);
        }
        Ok(Self { kinds })
    }

    fn matches(&self, kind: TratonEventKind) -> bool {
        self.kinds.contains(&kind)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventSubscriptionError {
    InvalidSubscriberId,
    DuplicateSubscriber,
    InvalidMailboxCapacity,
    EmptyFilter,
    TooManySubscribers,
}

impl fmt::Display for EventSubscriptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSubscriberId => formatter.write_str("invalid event subscriber identifier"),
            Self::DuplicateSubscriber => formatter.write_str("event subscriber is already registered"),
            Self::InvalidMailboxCapacity => formatter.write_str("event mailbox capacity must be between 1 and 4096"),
            Self::EmptyFilter => formatter.write_str("event subscription must select at least one event kind"),
            Self::TooManySubscribers => formatter.write_str("event router subscriber limit has been reached"),
        }
    }
}

impl std::error::Error for EventSubscriptionError {}

/// Non-blocking publication outcome across every matching subscriber.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EventDispatchReport {
    pub(crate) matched_subscribers: usize,
    pub(crate) delivered: usize,
    pub(crate) full_mailboxes: usize,
    pub(crate) disconnected_subscribers: usize,
}

struct EventSubscriber {
    filter: EventKindFilter,
    sender: SyncSender<TratonEvent>,
}

/// BGW-owned fan-out. The router invokes no consumer callbacks and never waits
/// for mailbox capacity; feature owners execute received work on their own host.
#[derive(Default)]
pub(crate) struct EventRouter {
    subscribers: BTreeMap<String, EventSubscriber>,
}

impl EventRouter {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Registers a bounded receiver for the selected normalized event kinds.
    /// The returned receiver may be moved to the owning feature thread.
    pub(crate) fn subscribe(
        &mut self,
        subscriber_id: &str,
        filter: EventKindFilter,
        capacity: usize,
    ) -> Result<Receiver<TratonEvent>, EventSubscriptionError> {
        if !valid_subscriber_id(subscriber_id) {
            return Err(EventSubscriptionError::InvalidSubscriberId);
        }
        if !(1..=MAX_EVENT_MAILBOX_CAPACITY).contains(&capacity) {
            return Err(EventSubscriptionError::InvalidMailboxCapacity);
        }
        if self.subscribers.contains_key(subscriber_id) {
            return Err(EventSubscriptionError::DuplicateSubscriber);
        }
        if self.subscribers.len() >= MAX_EVENT_SUBSCRIBERS {
            return Err(EventSubscriptionError::TooManySubscribers);
        }

        let (sender, receiver) = mpsc::sync_channel(capacity);
        self.subscribers.insert(
            subscriber_id.to_owned(),
            EventSubscriber { filter, sender },
        );
        Ok(receiver)
    }

    /// Publishes one already-normalized event to matching subscribers.
    ///
    /// try_send ensures a full mailbox is accounted for without blocking BGW
    /// or delaying another subscriber. Disconnected receivers are removed.
    pub(crate) fn publish(
        &mut self,
        event: &TratonEvent,
    ) -> Result<EventDispatchReport, EventQueueError> {
        event.validate()?;
        let mut report = EventDispatchReport::default();
        let mut disconnected = Vec::new();

        for (subscriber_id, subscriber) in &self.subscribers {
            if !subscriber.filter.matches(event.kind) {
                continue;
            }
            report.matched_subscribers += 1;
            match subscriber.sender.try_send(event.clone()) {
                Ok(()) => report.delivered += 1,
                Err(TrySendError::Full(_)) => report.full_mailboxes += 1,
                Err(TrySendError::Disconnected(_)) => {
                    report.disconnected_subscribers += 1;
                    disconnected.push(subscriber_id.clone());
                }
            }
        }

        for subscriber_id in disconnected {
            self.subscribers.remove(&subscriber_id);
        }

        Ok(report)
    }

    pub(crate) fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

fn valid_subscriber_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SUBSCRIBER_ID_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::{
        EventKindFilter, EventRouter, EventSubscriptionError, DEFAULT_EVENT_MAILBOX_CAPACITY,
    };
    use std::sync::mpsc::TryRecvError;
    use wl_traton::{EventQueueError, TratonEvent, TratonEventKind, TratonEventPayload};

    fn event(kind: TratonEventKind) -> TratonEvent {
        TratonEvent {
            kind,
            target_selector: "10-22h2".to_owned(),
            provider_id: "traton.winevent.desktop".to_owned(),
            monotonic_timestamp_ns: 10,
            generation: 1,
            object: None,
            payload: TratonEventPayload::None,
        }
    }

    fn filter(kinds: impl IntoIterator<Item = TratonEventKind>) -> EventKindFilter {
        EventKindFilter::only(kinds).unwrap()
    }

    #[test]
    fn subscribers_receive_only_the_kinds_they_selected() {
        let mut router = EventRouter::new();
        let desktop = router
            .subscribe(
                "wlde.desktop",
                filter([TratonEventKind::WindowGeometryChanged]),
                DEFAULT_EVENT_MAILBOX_CAPACITY,
            )
            .unwrap();
        let animation = router
            .subscribe(
                "animate.timeline",
                filter([
                    TratonEventKind::WindowActivated,
                    TratonEventKind::WindowShown,
                ]),
                DEFAULT_EVENT_MAILBOX_CAPACITY,
            )
            .unwrap();

        let report = router.publish(&event(TratonEventKind::WindowActivated)).unwrap();
        assert_eq!(report.matched_subscribers, 1);
        assert_eq!(report.delivered, 1);
        assert_eq!(report.full_mailboxes, 0);
        assert_eq!(animation.try_recv().unwrap().kind, TratonEventKind::WindowActivated);
        assert_eq!(desktop.try_recv(), Err(TryRecvError::Empty));

        let report = router
            .publish(&event(TratonEventKind::WindowGeometryChanged))
            .unwrap();
        assert_eq!(report.matched_subscribers, 1);
        assert_eq!(desktop.try_recv().unwrap().kind, TratonEventKind::WindowGeometryChanged);
        assert_eq!(animation.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn full_subscriber_mailbox_never_blocks_delivery_to_another_subscriber() {
        let mut router = EventRouter::new();
        let slow = router
            .subscribe(
                "wlde.desktop",
                filter([TratonEventKind::WindowShown]),
                1,
            )
            .unwrap();
        let fast = router
            .subscribe(
                "animate.timeline",
                filter([TratonEventKind::WindowShown]),
                2,
            )
            .unwrap();

        router.publish(&event(TratonEventKind::WindowShown)).unwrap();
        let report = router.publish(&event(TratonEventKind::WindowShown)).unwrap();
        assert_eq!(report.matched_subscribers, 2);
        assert_eq!(report.delivered, 1);
        assert_eq!(report.full_mailboxes, 1);
        assert_eq!(report.disconnected_subscribers, 0);
        assert_eq!(slow.try_recv().unwrap().kind, TratonEventKind::WindowShown);
        assert_eq!(fast.try_recv().unwrap().kind, TratonEventKind::WindowShown);
        assert_eq!(fast.try_recv().unwrap().kind, TratonEventKind::WindowShown);
    }

    #[test]
    fn disconnected_subscribers_are_removed_without_affecting_others() {
        let mut router = EventRouter::new();
        let disconnected = router
            .subscribe(
                "wlde.desktop",
                filter([TratonEventKind::WindowCreated]),
                1,
            )
            .unwrap();
        let live = router
            .subscribe(
                "bgw.diagnostics",
                filter([TratonEventKind::WindowCreated]),
                1,
            )
            .unwrap();
        drop(disconnected);

        let report = router.publish(&event(TratonEventKind::WindowCreated)).unwrap();
        assert_eq!(report.matched_subscribers, 2);
        assert_eq!(report.delivered, 1);
        assert_eq!(report.disconnected_subscribers, 1);
        assert_eq!(router.subscriber_count(), 1);
        assert_eq!(live.try_recv().unwrap().kind, TratonEventKind::WindowCreated);
    }

    #[test]
    fn rejects_invalid_subscription_ids_capacity_and_empty_filters() {
        let mut router = EventRouter::new();
        assert_eq!(
            router.subscribe("Wlde", filter([TratonEventKind::WindowShown]), 1).err(),
            Some(EventSubscriptionError::InvalidSubscriberId)
        );
        assert_eq!(
            router.subscribe("wlde.desktop", filter([TratonEventKind::WindowShown]), 0).err(),
            Some(EventSubscriptionError::InvalidMailboxCapacity)
        );
        assert_eq!(
            EventKindFilter::only([]).err(),
            Some(EventSubscriptionError::EmptyFilter)
        );
        router
            .subscribe("wlde.desktop", filter([TratonEventKind::WindowShown]), 1)
            .unwrap();
        assert_eq!(
            router.subscribe("wlde.desktop", filter([TratonEventKind::WindowShown]), 1).err(),
            Some(EventSubscriptionError::DuplicateSubscriber)
        );
    }

    #[test]
    fn router_rejects_an_invalid_event_envelope() {
        let mut router = EventRouter::new();
        router
            .subscribe("bgw.diagnostics", filter([TratonEventKind::WindowShown]), 1)
            .unwrap();
        let mut invalid = event(TratonEventKind::WindowShown);
        invalid.provider_id = "Traton.Invalid".to_owned();
        assert_eq!(
            router.publish(&invalid),
            Err(EventQueueError::InvalidIdentifier)
        );
    }
}
