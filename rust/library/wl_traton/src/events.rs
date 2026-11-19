//! Bounded semantic event envelopes and a host-owned queue.

use std::collections::VecDeque;
use std::fmt;

use crate::capability::CapabilityStatus;
use crate::objects::ObjectReference;

pub const DEFAULT_EVENT_QUEUE_CAPACITY: usize = 256;
pub const MAX_EVENT_QUEUE_CAPACITY: usize = 4096;

/// Normalized event names. They carry semantic meaning, not Win32 event IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TratonEventKind {
    ShellStarted,
    ShellStopped,
    ShellRestarted,
    ShellReady,
    ShellUnavailable,
    WindowCreated,
    WindowDestroyed,
    WindowShown,
    WindowHidden,
    WindowActivated,
    WindowDeactivated,
    WindowMoved,
    WindowResized,
    WindowMinimized,
    WindowRestored,
    MonitorTopologyChanged,
    DesktopInventoryInvalidated,
    CapabilityChanged,
    ProviderQuarantined,
}

impl TratonEventKind {
    fn coalescible(self) -> bool {
        matches!(
            self,
            Self::WindowMoved
                | Self::WindowResized
                | Self::MonitorTopologyChanged
                | Self::DesktopInventoryInvalidated
        )
    }
}

/// Bounded typed event data; callbacks should copy only the minimum needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TratonEventPayload {
    None,
    WindowBounds {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    },
    MonitorCount(u16),
    ShellReadiness {
        explorer_process: bool,
        desktop_view: bool,
        taskbar: bool,
    },
    Capability {
        capability_id: String,
        status: CapabilityStatus,
    },
}

/// One event from a provider generation. Timestamp units are monotonic nanoseconds
/// selected by the host; Traton does not read wall-clock time inside callbacks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TratonEvent {
    pub kind: TratonEventKind,
    pub target_selector: String,
    pub provider_id: String,
    pub monotonic_timestamp_ns: u64,
    pub generation: u64,
    pub object: Option<ObjectReference>,
    pub payload: TratonEventPayload,
}

impl TratonEvent {
    pub fn validate(&self) -> Result<(), EventQueueError> {
        validate_identifier(&self.target_selector, 64)?;
        validate_identifier(&self.provider_id, 64)?;
        if let TratonEventPayload::Capability { capability_id, .. } = &self.payload {
            validate_identifier(capability_id, 128)?;
        }
        Ok(())
    }

    fn same_coalescing_key(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.provider_id == other.provider_id
            && self.target_selector == other.target_selector
            && self.object == other.object
            && self.generation == other.generation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventQueuePushResult {
    Queued,
    Coalesced,
    DroppedCoalescible,
    StaleGeneration,
    QueueSaturated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventQueueError {
    InvalidCapacity,
    InvalidIdentifier,
}

impl fmt::Display for EventQueueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCapacity => formatter.write_str("event queue capacity must be between 1 and 4096"),
            Self::InvalidIdentifier => formatter.write_str("event envelope contains an invalid or oversized identifier"),
        }
    }
}

impl std::error::Error for EventQueueError {}

/// Single-owner bounded queue. BGW owns pump scheduling, thread handoff and routing.
#[derive(Clone, Debug)]
pub struct BoundedEventQueue {
    queue: VecDeque<TratonEvent>,
    capacity: usize,
    generation: u64,
    dropped_events: u64,
    stale_events: u64,
}

impl BoundedEventQueue {
    pub fn new(capacity: usize, generation: u64) -> Result<Self, EventQueueError> {
        if capacity == 0 || capacity > MAX_EVENT_QUEUE_CAPACITY {
            return Err(EventQueueError::InvalidCapacity);
        }
        Ok(Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
            generation,
            dropped_events: 0,
            stale_events: 0,
        })
    }

    pub fn standard(generation: u64) -> Self {
        Self::new(DEFAULT_EVENT_QUEUE_CAPACITY, generation)
            .expect("the default event queue capacity is valid")
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn dropped_events(&self) -> u64 {
        self.dropped_events
    }

    pub fn stale_events(&self) -> u64 {
        self.stale_events
    }

    /// Advances a host/provider generation and discards every queued old event.
    pub fn advance_generation(&mut self, generation: u64) {
        self.generation = generation;
        self.dropped_events = self.dropped_events.saturating_add(self.queue.len() as u64);
        self.queue.clear();
    }

    pub fn push(&mut self, event: TratonEvent) -> Result<EventQueuePushResult, EventQueueError> {
        event.validate()?;
        if event.generation != self.generation {
            self.stale_events = self.stale_events.saturating_add(1);
            return Ok(EventQueuePushResult::StaleGeneration);
        }

        if event.kind.coalescible() {
            if let Some(existing) = self
                .queue
                .iter_mut()
                .rev()
                .find(|existing| existing.same_coalescing_key(&event))
            {
                *existing = event;
                return Ok(EventQueuePushResult::Coalesced);
            }
        }

        if self.queue.len() == self.capacity {
            if event.kind.coalescible() {
                self.dropped_events = self.dropped_events.saturating_add(1);
                return Ok(EventQueuePushResult::DroppedCoalescible);
            }

            if let Some(index) = self
                .queue
                .iter()
                .position(|queued| queued.kind.coalescible())
            {
                self.queue.remove(index);
                self.dropped_events = self.dropped_events.saturating_add(1);
            } else {
                self.dropped_events = self.dropped_events.saturating_add(1);
                return Ok(EventQueuePushResult::QueueSaturated);
            }
        }

        self.queue.push_back(event);
        Ok(EventQueuePushResult::Queued)
    }

    pub fn pop(&mut self) -> Option<TratonEvent> {
        self.queue.pop_front()
    }
}

fn validate_identifier(value: &str, max_len: usize) -> Result<(), EventQueueError> {
    if value.is_empty()
        || value.len() > max_len
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b'_')
        })
    {
        return Err(EventQueueError::InvalidIdentifier);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BoundedEventQueue, EventQueuePushResult, TratonEvent, TratonEventKind, TratonEventPayload,
    };

    fn event(kind: TratonEventKind, generation: u64, object: Option<&str>) -> TratonEvent {
        TratonEvent {
            kind,
            target_selector: "10-22h2".to_owned(),
            provider_id: "traton.test".to_owned(),
            monotonic_timestamp_ns: 10,
            generation,
            object: object.map(|value| crate::objects::ObjectReference::parse(value).unwrap()),
            payload: TratonEventPayload::None,
        }
    }

    #[test]
    fn coalesces_only_matching_high_frequency_events() {
        let mut queue = BoundedEventQueue::new(2, 7).unwrap();
        assert_eq!(
            queue
                .push(event(
                    TratonEventKind::WindowMoved,
                    7,
                    Some("running-window:window-1")
                ))
                .unwrap(),
            EventQueuePushResult::Queued
        );
        let mut newest = event(
            TratonEventKind::WindowMoved,
            7,
            Some("running-window:window-1"),
        );
        newest.monotonic_timestamp_ns = 99;
        assert_eq!(queue.push(newest).unwrap(), EventQueuePushResult::Coalesced);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop().unwrap().monotonic_timestamp_ns, 99);
    }

    #[test]
    fn stale_generation_events_are_rejected() {
        let mut queue = BoundedEventQueue::new(2, 8).unwrap();
        assert_eq!(
            queue
                .push(event(TratonEventKind::WindowCreated, 7, None))
                .unwrap(),
            EventQueuePushResult::StaleGeneration
        );
        assert_eq!(queue.stale_events(), 1);
    }

    #[test]
    fn discrete_lifecycle_events_displace_coalescible_events_when_full() {
        let mut queue = BoundedEventQueue::new(1, 1).unwrap();
        queue
            .push(event(TratonEventKind::WindowMoved, 1, None))
            .unwrap();
        assert_eq!(
            queue
                .push(event(TratonEventKind::ShellRestarted, 1, None))
                .unwrap(),
            EventQueuePushResult::Queued
        );
        assert_eq!(queue.pop().unwrap().kind, TratonEventKind::ShellRestarted);
        assert_eq!(queue.dropped_events(), 1);
    }

    #[test]
    fn queue_never_discards_a_discrete_event_for_another_discrete_event() {
        let mut queue = BoundedEventQueue::new(1, 1).unwrap();
        queue
            .push(event(TratonEventKind::ShellStopped, 1, None))
            .unwrap();
        assert_eq!(
            queue
                .push(event(TratonEventKind::ShellStarted, 1, None))
                .unwrap(),
            EventQueuePushResult::QueueSaturated
        );
        assert_eq!(queue.pop().unwrap().kind, TratonEventKind::ShellStopped);
    }
}
