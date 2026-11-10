//! Portable admission/accounting core for WinLook-owned CPU work.
//!
//! This library does not create threads, schedule OS work, measure CPU time,
//! or control unrelated processes. The host supplies measurements and runs
//! admitted work. Access to a Governor requires external synchronization when
//! shared between threads.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Relative importance used when applying the reserved critical headroom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkClass {
    Critical,
    Interactive,
    Foreground,
    Background,
    Idle,
}

/// Per-accounting-window limits. Costs are caller-defined abstract CPU work units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub window_budget: u64,
    pub critical_reserve: u64,
    pub max_in_flight: usize,
}

impl Policy {
    pub fn validate(self) -> Result<Self, Error> {
        if self.window_budget == 0
            || self.critical_reserve > self.window_budget
            || self.max_in_flight == 0
        {
            return Err(Error::InvalidPolicy);
        }
        Ok(self)
    }
}

/// Opaque handle to one admitted work item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ticket(u64);

impl Ticket {
    pub fn id(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidPolicy,
    InvalidEstimate,
    BudgetExceeded,
    ConcurrencyLimit,
    UnknownTicket,
    TicketExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressure {
    Normal,
    Constrained,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Completion {
    pub estimated_units: u64,
    pub actual_units: u64,
    pub overrun: bool,
}

#[derive(Clone, Copy, Debug)]
struct Reservation {
    estimate: u64,
}

/// Deterministic CPU work admission core. It is not an OS scheduler.
#[derive(Debug)]
pub struct Governor {
    policy: Policy,
    spent: u64,
    reserved: u64,
    next_ticket: u64,
    active: BTreeMap<u64, Reservation>,
}

impl Governor {
    pub fn new(policy: Policy) -> Result<Self, Error> {
        Ok(Self {
            policy: policy.validate()?,
            spent: 0,
            reserved: 0,
            next_ticket: 1,
            active: BTreeMap::new(),
        })
    }

    pub fn policy(&self) -> Policy {
        self.policy
    }
    pub fn spent_units(&self) -> u64 {
        self.spent
    }
    pub fn reserved_units(&self) -> u64 {
        self.reserved
    }
    pub fn in_flight(&self) -> usize {
        self.active.len()
    }

    pub fn available_for(&self, class: WorkClass) -> u64 {
        let limit = if class == WorkClass::Critical {
            self.policy.window_budget
        } else {
            self.policy.window_budget - self.policy.critical_reserve
        };
        limit.saturating_sub(self.spent.saturating_add(self.reserved))
    }

    pub fn try_acquire(&mut self, class: WorkClass, estimate: u64) -> Result<Ticket, Error> {
        if estimate == 0 {
            return Err(Error::InvalidEstimate);
        }
        if self.active.len() >= self.policy.max_in_flight {
            return Err(Error::ConcurrencyLimit);
        }
        if estimate > self.available_for(class) {
            return Err(Error::BudgetExceeded);
        }
        let id = self.next_ticket;
        self.next_ticket = self
            .next_ticket
            .checked_add(1)
            .ok_or(Error::TicketExhausted)?;
        self.reserved += estimate;
        self.active.insert(id, Reservation { estimate });
        Ok(Ticket(id))
    }

    /// Records measured CPU work units. An overrun is reported, not hidden.
    pub fn complete(&mut self, ticket: Ticket, actual: u64) -> Result<Completion, Error> {
        let reservation = self.active.remove(&ticket.0).ok_or(Error::UnknownTicket)?;
        self.reserved -= reservation.estimate;
        self.spent = self.spent.saturating_add(actual);
        Ok(Completion {
            estimated_units: reservation.estimate,
            actual_units: actual,
            overrun: actual > reservation.estimate,
        })
    }

    /// Cancels work before it runs and refunds its reservation exactly once.
    pub fn cancel(&mut self, ticket: Ticket) -> Result<u64, Error> {
        let reservation = self.active.remove(&ticket.0).ok_or(Error::UnknownTicket)?;
        self.reserved -= reservation.estimate;
        Ok(reservation.estimate)
    }

    /// Starts a new accounting window. Live reservations remain reserved.
    pub fn reset_window(&mut self) {
        self.spent = 0;
    }

    pub fn pressure(&self) -> Pressure {
        let used = self.spent.saturating_add(self.reserved);
        if used >= self.policy.window_budget {
            Pressure::Critical
        } else if used >= self.policy.window_budget - self.policy.window_budget / 5 {
            Pressure::Constrained
        } else {
            Pressure::Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn governor() -> Governor {
        Governor::new(Policy {
            window_budget: 100,
            critical_reserve: 20,
            max_in_flight: 2,
        })
        .unwrap()
    }

    #[test]
    fn reserves_budget_and_protects_critical_headroom() {
        let mut g = governor();
        assert_eq!(g.available_for(WorkClass::Background), 80);
        let t = g.try_acquire(WorkClass::Background, 80).unwrap();
        assert_eq!(
            g.try_acquire(WorkClass::Interactive, 1),
            Err(Error::BudgetExceeded)
        );
        let critical = g.try_acquire(WorkClass::Critical, 20).unwrap();
        assert_eq!(critical.id(), 2);
        let completion = g.complete(t, 70).unwrap();
        assert!(!completion.overrun);
        assert_eq!(g.spent_units(), 70);
        g.complete(critical, 15).unwrap();
    }

    #[test]
    fn concurrency_and_duplicate_ticket_are_bounded() {
        let mut g = governor();
        let a = g.try_acquire(WorkClass::Interactive, 1).unwrap();
        let b = g.try_acquire(WorkClass::Background, 1).unwrap();
        assert_eq!(
            g.try_acquire(WorkClass::Idle, 1),
            Err(Error::ConcurrencyLimit)
        );
        g.cancel(a).unwrap();
        assert_eq!(g.cancel(a), Err(Error::UnknownTicket));
        g.complete(b, 2).unwrap();
    }

    #[test]
    fn completion_reports_overrun_and_window_reset_keeps_live_reservations() {
        let mut g = governor();
        let t = g.try_acquire(WorkClass::Interactive, 10).unwrap();
        let c = g.complete(t, 30).unwrap();
        assert!(c.overrun);
        assert_eq!(g.pressure(), Pressure::Normal);
        let live = g.try_acquire(WorkClass::Interactive, 10).unwrap();
        g.reset_window();
        assert_eq!(g.spent_units(), 0);
        assert_eq!(g.reserved_units(), 10);
        g.cancel(live).unwrap();
    }

    #[test]
    fn rejects_invalid_policy_and_zero_estimate() {
        assert_eq!(
            Governor::new(Policy {
                window_budget: 1,
                critical_reserve: 2,
                max_in_flight: 1
            })
            .err(),
            Some(Error::InvalidPolicy)
        );
        assert_eq!(
            governor().try_acquire(WorkClass::Idle, 0),
            Err(Error::InvalidEstimate)
        );
    }
}
