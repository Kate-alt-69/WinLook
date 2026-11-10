//! Portable GPU memory/submission admission core for WinLook.
//!
//! This library does not allocate graphics resources, submit commands, query
//! DXGI or guarantee residency. A platform adapter supplies OS budget snapshots
//! and owns actual graphics objects. Synchronize shared access externally.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkClass { Critical, Interactive, Foreground, Background, Idle }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub memory_budget_bytes: u64,
    pub critical_reserve_bytes: u64,
    pub max_in_flight_submissions: usize,
}

impl Policy {
    pub fn validate(self) -> Result<Self, Error> {
        if self.memory_budget_bytes == 0 || self.critical_reserve_bytes > self.memory_budget_bytes
            || self.max_in_flight_submissions == 0 { return Err(Error::InvalidPolicy); }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceTicket(u64);
impl ResourceTicket { pub fn id(self) -> u64 { self.0 } }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubmissionTicket(u64);
impl SubmissionTicket { pub fn id(self) -> u64 { self.0 } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidPolicy, InvalidEstimate, BudgetExceeded, QueueFull, UnknownResource, UnknownSubmission, TicketExhausted }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressure { Normal, Constrained, Critical }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Allocation { bytes: u64 }

/// Resource and submission admission. Actual GPU work remains owned by a renderer.
#[derive(Debug)]
pub struct Governor {
    policy: Policy,
    next_id: u64,
    allocations: BTreeMap<u64, Allocation>,
    allocated_bytes: u64,
    submissions: BTreeMap<u64, ()>,
}

impl Governor {
    pub fn new(policy: Policy) -> Result<Self, Error> {
        Ok(Self { policy: policy.validate()?, next_id: 1, allocations: BTreeMap::new(),
            allocated_bytes: 0, submissions: BTreeMap::new() })
    }

    pub fn policy(&self) -> Policy { self.policy }
    pub fn allocated_bytes(&self) -> u64 { self.allocated_bytes }
    pub fn in_flight_submissions(&self) -> usize { self.submissions.len() }

    fn next_ticket(&mut self) -> Result<u64, Error> {
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).ok_or(Error::TicketExhausted)?;
        Ok(id)
    }

    pub fn available_for(&self, class: WorkClass) -> u64 {
        let limit = if class == WorkClass::Critical { self.policy.memory_budget_bytes }
            else { self.policy.memory_budget_bytes - self.policy.critical_reserve_bytes };
        limit.saturating_sub(self.allocated_bytes)
    }

    pub fn try_reserve(&mut self, class: WorkClass, estimated_bytes: u64) -> Result<ResourceTicket, Error> {
        if estimated_bytes == 0 { return Err(Error::InvalidEstimate); }
        if estimated_bytes > self.available_for(class) { return Err(Error::BudgetExceeded); }
        let id = self.next_ticket()?;
        self.allocated_bytes = self.allocated_bytes.saturating_add(estimated_bytes);
        self.allocations.insert(id, Allocation { bytes: estimated_bytes });
        Ok(ResourceTicket(id))
    }

    /// Releases a WinLook-owned allocation lease. The renderer must release the
    /// actual graphics resource before calling this method.
    pub fn release(&mut self, ticket: ResourceTicket) -> Result<u64, Error> {
        let allocation = self.allocations.remove(&ticket.0).ok_or(Error::UnknownResource)?;
        self.allocated_bytes -= allocation.bytes;
        Ok(allocation.bytes)
    }

    pub fn begin_submission(&mut self) -> Result<SubmissionTicket, Error> {
        if self.submissions.len() >= self.policy.max_in_flight_submissions { return Err(Error::QueueFull); }
        let id = self.next_ticket()?;
        self.submissions.insert(id, ());
        Ok(SubmissionTicket(id))
    }

    /// Call only after the renderer's fence/completion signal confirms completion.
    pub fn complete_submission(&mut self, ticket: SubmissionTicket) -> Result<(), Error> {
        self.submissions.remove(&ticket.0).map(|_| ()).ok_or(Error::UnknownSubmission)
    }

    /// Applies an OS-reported budget change; existing allocations are not destroyed.
    pub fn update_budget(&mut self, budget_bytes: u64, critical_reserve_bytes: u64) -> Result<(), Error> {
        if budget_bytes == 0 || critical_reserve_bytes > budget_bytes { return Err(Error::InvalidPolicy); }
        self.policy.memory_budget_bytes = budget_bytes;
        self.policy.critical_reserve_bytes = critical_reserve_bytes;
        Ok(())
    }

    pub fn pressure(&self) -> Pressure {
        if self.allocated_bytes >= self.policy.memory_budget_bytes { Pressure::Critical }
        else if self.allocated_bytes >= self.policy.memory_budget_bytes - self.policy.memory_budget_bytes / 5 { Pressure::Constrained }
        else { Pressure::Normal }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn governor() -> Governor {
        Governor::new(Policy { memory_budget_bytes: 1_000, critical_reserve_bytes: 200, max_in_flight_submissions: 2 }).unwrap()
    }

    #[test]
    fn protects_critical_memory_headroom() {
        let mut g = governor();
        let normal = g.try_reserve(WorkClass::Background, 800).unwrap();
        assert_eq!(g.try_reserve(WorkClass::Interactive, 1), Err(Error::BudgetExceeded));
        let critical = g.try_reserve(WorkClass::Critical, 200).unwrap();
        assert_eq!(g.allocated_bytes(), 1_000);
        assert_eq!(g.pressure(), Pressure::Critical);
        g.release(normal).unwrap();
        g.release(critical).unwrap();
        assert_eq!(g.allocated_bytes(), 0);
    }

    #[test]
    fn submissions_are_bounded_and_stale_tickets_rejected() {
        let mut g = governor();
        let a = g.begin_submission().unwrap();
        let b = g.begin_submission().unwrap();
        assert_eq!(g.begin_submission(), Err(Error::QueueFull));
        g.complete_submission(a).unwrap();
        assert_eq!(g.complete_submission(a), Err(Error::UnknownSubmission));
        g.complete_submission(b).unwrap();
    }

    #[test]
    fn dynamic_budget_shrink_does_not_corrupt_existing_allocations() {
        let mut g = governor();
        let lease = g.try_reserve(WorkClass::Interactive, 600).unwrap();
        g.update_budget(500, 100).unwrap();
        assert_eq!(g.pressure(), Pressure::Critical);
        assert_eq!(g.try_reserve(WorkClass::Critical, 1), Err(Error::BudgetExceeded));
        assert_eq!(g.release(lease).unwrap(), 600);
        assert_eq!(g.allocated_bytes(), 0);
    }

    #[test]
    fn rejects_invalid_inputs_and_duplicate_release() {
        assert_eq!(Governor::new(Policy { memory_budget_bytes: 10, critical_reserve_bytes: 11, max_in_flight_submissions: 1 }).err(), Some(Error::InvalidPolicy));
        let mut g = governor();
        assert_eq!(g.try_reserve(WorkClass::Idle, 0), Err(Error::InvalidEstimate));
        let ticket = g.try_reserve(WorkClass::Idle, 1).unwrap();
        g.release(ticket).unwrap();
        assert_eq!(g.release(ticket), Err(Error::UnknownResource));
    }
}
