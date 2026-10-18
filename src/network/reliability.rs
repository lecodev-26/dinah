//! Bounded application-level retransmission bookkeeping.
//!
//! TCP already retransmits lost network segments. This module is for protocols
//! that explicitly need message acknowledgements/retries above a transport; it
//! does not inspect or control operating-system packet loss.
use core::fmt;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingMessage<T> {
    pub sequence: u64,
    pub payload: T,
    pub last_sent_tick: u64,
    pub attempts: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliabilityError {
    ZeroCapacity,
    Full,
    SequenceExhausted,
    ZeroRetryInterval,
}
impl fmt::Display for ReliabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapacity => f.write_str("reliable outbox capacity must be non-zero"),
            Self::Full => f.write_str("reliable outbox is full"),
            Self::SequenceExhausted => f.write_str("reliable message sequence is exhausted"),
            Self::ZeroRetryInterval => f.write_str("retry interval must be non-zero"),
        }
    }
}
impl std::error::Error for ReliabilityError {}

/// A bounded outbox that retains messages until acknowledged or explicitly expired.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReliableOutbox<T> {
    pending: VecDeque<PendingMessage<T>>,
    capacity: usize,
    next_sequence: u64,
}
impl<T> ReliableOutbox<T> {
    pub fn new(capacity: usize) -> Result<Self, ReliabilityError> {
        if capacity == 0 {
            return Err(ReliabilityError::ZeroCapacity);
        }
        Ok(Self {
            pending: VecDeque::with_capacity(capacity),
            capacity,
            next_sequence: 1,
        })
    }
    pub const fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn len(&self) -> usize {
        self.pending.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub fn pending(&self) -> impl ExactSizeIterator<Item = &PendingMessage<T>> {
        self.pending.iter()
    }
    pub fn enqueue(&mut self, payload: T, tick: u64) -> Result<u64, ReliabilityError> {
        if self.pending.len() == self.capacity {
            return Err(ReliabilityError::Full);
        }
        let sequence = self.next_sequence;
        let next = sequence
            .checked_add(1)
            .ok_or(ReliabilityError::SequenceExhausted)?;
        self.pending.push_back(PendingMessage {
            sequence,
            payload,
            last_sent_tick: tick,
            attempts: 1,
        });
        self.next_sequence = next;
        Ok(sequence)
    }
    /// Acknowledges all messages through a contiguous sequence number.
    pub fn acknowledge(&mut self, through: u64) -> usize {
        let before = self.pending.len();
        self.pending.retain(|message| message.sequence > through);
        before - self.pending.len()
    }
    /// Marks due messages for retransmission and returns their sequence numbers.
    /// Attempts saturate rather than wrapping. The caller performs actual sends.
    pub fn due_retries(
        &mut self,
        now_tick: u64,
        retry_interval: u64,
        max_attempts: u32,
    ) -> Result<Vec<u64>, ReliabilityError> {
        if retry_interval == 0 {
            return Err(ReliabilityError::ZeroRetryInterval);
        }
        let mut due = Vec::new();
        for message in &mut self.pending {
            if message.attempts < max_attempts
                && now_tick.saturating_sub(message.last_sent_tick) >= retry_interval
            {
                message.last_sent_tick = now_tick;
                message.attempts = message.attempts.saturating_add(1);
                due.push(message.sequence);
            }
        }
        Ok(due)
    }
}
