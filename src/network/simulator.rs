//! Deterministic tick-based network simulator for tests and experiments.

#[derive(Clone, Debug, PartialEq, Eq)]
struct Scheduled<T> {
    deliver_at: u64,
    sequence: u64,
    payload: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulatorError {
    SequenceExhausted,
}

/// Deterministic simulated link. A `drop` decision is supplied by the caller,
/// avoiding hidden randomness and making packet-loss scenarios reproducible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkSimulator<T> {
    scheduled: Vec<Scheduled<T>>,
    next_sequence: u64,
    dropped: u64,
    delivered: u64,
}
impl<T> Default for NetworkSimulator<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> NetworkSimulator<T> {
    pub fn new() -> Self {
        Self {
            scheduled: Vec::new(),
            next_sequence: 0,
            dropped: 0,
            delivered: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.scheduled.len()
    }
    pub fn is_empty(&self) -> bool {
        self.scheduled.is_empty()
    }
    pub const fn dropped_count(&self) -> u64 {
        self.dropped
    }
    pub const fn delivered_count(&self) -> u64 {
        self.delivered
    }
    pub fn send(
        &mut self,
        now_tick: u64,
        latency_ticks: u64,
        payload: T,
        drop: bool,
    ) -> Result<Option<u64>, SimulatorError> {
        let sequence = self.next_sequence;
        self.next_sequence = sequence
            .checked_add(1)
            .ok_or(SimulatorError::SequenceExhausted)?;
        if drop {
            self.dropped = self.dropped.saturating_add(1);
            return Ok(None);
        }
        self.scheduled.push(Scheduled {
            deliver_at: now_tick.saturating_add(latency_ticks),
            sequence,
            payload,
        });
        Ok(Some(sequence))
    }
    /// Delivers packets due at the current logical tick of a Simulation.
    /// This is an explicit pull operation; it never mutates simulation state.
    pub fn receive_due_for_simulation<S>(
        &mut self,
        simulation: &crate::simulation::Simulation<S>,
    ) -> Vec<T> {
        self.receive_due(simulation.clock().tick())
    }

    /// Delivers all packets whose due tick is at or before `now_tick`, ordered by due tick then send order.
    pub fn receive_due(&mut self, now_tick: u64) -> Vec<T> {
        let mut result = Vec::new();
        self.scheduled
            .sort_by_key(|packet| (packet.deliver_at, packet.sequence));
        let split = self
            .scheduled
            .partition_point(|packet| packet.deliver_at <= now_tick);
        let due: Vec<_> = self.scheduled.drain(..split).collect();
        result.extend(due.into_iter().map(|packet| packet.payload));
        self.delivered = self.delivered.saturating_add(result.len() as u64);
        result
    }
}
