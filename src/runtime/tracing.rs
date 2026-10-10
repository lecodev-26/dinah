use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// A structured Runtime lifecycle event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceEvent {
    TaskSubmitted,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    CancellationRequested,
    CancellationAcknowledged,
}

/// Error from recording an event, retaining the event for caller recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceRecordError {
    Full(TraceEvent),
    Poisoned(TraceEvent),
}

impl TraceRecordError {
    #[must_use]
    pub const fn into_event(self) -> TraceEvent {
        match self {
            Self::Full(event) | Self::Poisoned(event) => event,
        }
    }
}

impl core::fmt::Display for TraceRecordError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Full(_) => f.write_str("trace buffer is full"),
            Self::Poisoned(_) => f.write_str("trace buffer lock is poisoned"),
        }
    }
}

impl std::error::Error for TraceRecordError {}

/// Bounded, thread-safe in-memory trace buffer.
///
/// Recording never overwrites older events. When full, the rejected event is
/// returned to the caller. This buffer is opt-in and is not wired into scheduler
/// execution automatically.
#[derive(Debug)]
pub struct TraceBuffer {
    capacity: NonZeroUsize,
    events: Mutex<VecDeque<TraceEvent>>,
}

impl TraceBuffer {
    #[must_use]
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            events: Mutex::new(VecDeque::with_capacity(capacity.get())),
        }
    }

    /// Records an event or returns it unchanged when recording is impossible.
    pub fn record(&self, event: TraceEvent) -> Result<(), TraceRecordError> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| TraceRecordError::Poisoned(event))?;
        if events.len() == self.capacity.get() {
            return Err(TraceRecordError::Full(event));
        }
        events.push_back(event);
        Ok(())
    }

    /// Copies the currently buffered events in recording order.
    pub fn snapshot(&self) -> Result<Vec<TraceEvent>, TraceRecordError> {
        self.events
            .lock()
            .map(|events| events.iter().copied().collect())
            .map_err(|_| {
                // A snapshot has no event to preserve; use a representative event to
                // keep this API's error type simple and consistent.
                TraceRecordError::Poisoned(TraceEvent::TaskFailed)
            })
    }

    /// Removes and returns all buffered events in recording order.
    pub fn drain(&self) -> Result<Vec<TraceEvent>, TraceRecordError> {
        self.events
            .lock()
            .map(|mut events| events.drain(..).collect())
            .map_err(|_| TraceRecordError::Poisoned(TraceEvent::TaskFailed))
    }

    #[must_use]
    pub const fn capacity(&self) -> NonZeroUsize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::{TraceBuffer, TraceEvent, TraceRecordError};
    use std::num::NonZeroUsize;

    #[test]
    fn records_and_drains_events_in_order() {
        let buffer = TraceBuffer::new(NonZeroUsize::new(2).unwrap());
        buffer.record(TraceEvent::TaskSubmitted).unwrap();
        buffer.record(TraceEvent::TaskStarted).unwrap();
        assert_eq!(
            buffer.snapshot().unwrap(),
            vec![TraceEvent::TaskSubmitted, TraceEvent::TaskStarted]
        );
        assert_eq!(
            buffer.drain().unwrap(),
            vec![TraceEvent::TaskSubmitted, TraceEvent::TaskStarted]
        );
        assert!(buffer.snapshot().unwrap().is_empty());
    }

    #[test]
    fn full_buffer_returns_event_without_overwriting_existing_events() {
        let buffer = TraceBuffer::new(NonZeroUsize::new(1).unwrap());
        buffer.record(TraceEvent::TaskSubmitted).unwrap();
        assert_eq!(
            buffer.record(TraceEvent::TaskFailed),
            Err(TraceRecordError::Full(TraceEvent::TaskFailed))
        );
        assert_eq!(buffer.snapshot().unwrap(), vec![TraceEvent::TaskSubmitted]);
    }

    #[test]
    fn concurrent_recording_is_safe_and_bounded() {
        let buffer = std::sync::Arc::new(TraceBuffer::new(NonZeroUsize::new(800).unwrap()));
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let buffer = std::sync::Arc::clone(&buffer);
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        let _ = buffer.record(TraceEvent::TaskCompleted);
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(buffer.snapshot().unwrap().len(), 800);
    }
}
