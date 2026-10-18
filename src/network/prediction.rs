//! Bounded client-side prediction and authoritative reconciliation primitives.
//!
//! This module is transport-independent. It does not send packets, choose a
//! clock, or promise cross-platform bitwise determinism. Callers provide the
//! same state transition function for prediction and replay.

use core::fmt;
use std::collections::VecDeque;

/// One client input retained until the server acknowledges its sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputFrame<I> {
    /// Monotonically increasing identifier, starting at one.
    pub sequence: u64,
    /// Logical simulation tick at which the input was created.
    pub tick: u64,
    /// Domain-specific input payload.
    pub input: I,
}

/// Authoritative server state and the last client input processed by the server.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeSnapshot<S> {
    /// Logical server tick represented by this state.
    pub tick: u64,
    /// Highest contiguous input sequence processed for this client (zero means none).
    pub last_processed_input: u64,
    /// Authoritative simulation state at `tick`.
    pub state: S,
}

/// Errors returned by prediction history operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictionError {
    /// A history must retain at least one input.
    ZeroCapacity,
    /// The sequence counter cannot advance without wrapping.
    SequenceExhausted,
    /// The configured pending-input history is full.
    HistoryFull,
    /// A snapshot acknowledges an input sequence never issued by this client.
    InvalidAcknowledgement,
}

impl fmt::Display for PredictionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapacity => f.write_str("prediction history capacity must be non-zero"),
            Self::SequenceExhausted => f.write_str("prediction input sequence is exhausted"),
            Self::HistoryFull => f.write_str("prediction input history is full"),
            Self::InvalidAcknowledgement => {
                f.write_str("authoritative snapshot acknowledges an unknown input sequence")
            }
        }
    }
}

impl std::error::Error for PredictionError {}

/// Result of applying an authoritative snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconcileOutcome {
    /// Snapshot was accepted and pending inputs were replayed.
    Applied { replayed_inputs: usize },
    /// Snapshot was older than the latest accepted snapshot and was ignored.
    StaleSnapshot,
}

/// Bounded input history and predicted state for one client-controlled entity.
///
/// Prediction is provisional. The authoritative server remains responsible
/// for validating inputs and deciding final state. A full history rejects new
/// inputs rather than silently dropping unacknowledged inputs.
pub struct PredictionBuffer<S, I> {
    state: S,
    pending: VecDeque<InputFrame<I>>,
    capacity: usize,
    last_issued: u64,
    last_acknowledged: u64,
    last_snapshot_tick: Option<u64>,
}

impl<S, I> PredictionBuffer<S, I> {
    /// Creates a prediction buffer with a bounded pending-input history.
    pub fn new(initial_state: S, capacity: usize) -> Result<Self, PredictionError> {
        if capacity == 0 {
            return Err(PredictionError::ZeroCapacity);
        }
        Ok(Self {
            state: initial_state,
            pending: VecDeque::with_capacity(capacity),
            capacity,
            last_issued: 0,
            last_acknowledged: 0,
            last_snapshot_tick: None,
        })
    }

    /// Returns the current predicted state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Returns the highest input sequence issued so far.
    pub const fn last_issued_sequence(&self) -> u64 {
        self.last_issued
    }

    /// Returns the highest input sequence acknowledged by an accepted snapshot.
    pub const fn last_acknowledged_sequence(&self) -> u64 {
        self.last_acknowledged
    }

    /// Returns the number of unacknowledged inputs retained for replay.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Returns the configured maximum number of pending inputs.
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Iterates over pending inputs in sequence order.
    pub fn pending_inputs(&self) -> impl ExactSizeIterator<Item = &InputFrame<I>> {
        self.pending.iter()
    }

    /// Applies an input immediately and stores it for possible replay.
    ///
    /// Capacity and sequence availability are checked before state mutation.
    /// `step` should be a deterministic, side-effect-free state transition for
    /// a given state/input/tick. This method does not transmit the input.
    pub fn predict(
        &mut self,
        tick: u64,
        input: I,
        mut step: impl FnMut(&mut S, &I, u64),
    ) -> Result<u64, PredictionError> {
        if self.pending.len() >= self.capacity {
            return Err(PredictionError::HistoryFull);
        }
        let sequence = self
            .last_issued
            .checked_add(1)
            .ok_or(PredictionError::SequenceExhausted)?;

        step(&mut self.state, &input, tick);
        self.pending.push_back(InputFrame {
            sequence,
            tick,
            input,
        });
        self.last_issued = sequence;
        Ok(sequence)
    }

    /// Replaces predicted state with an authoritative snapshot and replays
    /// inputs newer than the server's acknowledgement in original order.
    ///
    /// Snapshots older than the latest accepted server tick or acknowledgement
    /// are ignored. Acknowledgements beyond the last issued sequence are
    /// rejected without mutating state or history.
    pub fn reconcile(
        &mut self,
        snapshot: AuthoritativeSnapshot<S>,
        mut step: impl FnMut(&mut S, &I, u64),
    ) -> Result<ReconcileOutcome, PredictionError> {
        if snapshot.last_processed_input > self.last_issued {
            return Err(PredictionError::InvalidAcknowledgement);
        }
        if self
            .last_snapshot_tick
            .is_some_and(|tick| snapshot.tick < tick)
            || snapshot.last_processed_input < self.last_acknowledged
        {
            return Ok(ReconcileOutcome::StaleSnapshot);
        }

        let acknowledged = snapshot.last_processed_input;
        self.pending.retain(|frame| frame.sequence > acknowledged);
        self.state = snapshot.state;
        let replayed_inputs = self.pending.len();
        for frame in &self.pending {
            step(&mut self.state, &frame.input, frame.tick);
        }
        self.last_acknowledged = acknowledged;
        self.last_snapshot_tick = Some(snapshot.tick);
        Ok(ReconcileOutcome::Applied { replayed_inputs })
    }
}
