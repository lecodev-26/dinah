use core::sync::atomic::{AtomicU64, Ordering};

/// Monotonic, thread-safe counters for observable Runtime events.
///
/// Counters record events explicitly reported by the caller; they do not
/// instrument task execution automatically. They saturate at `u64::MAX` rather
/// than wrapping and producing misleading smaller values.
#[derive(Debug, Default)]
pub struct RuntimeMetrics {
    submitted: AtomicU64,
    started: AtomicU64,
    completed: AtomicU64,
    failed: AtomicU64,
    cancellation_requested: AtomicU64,
    cancellation_acknowledged: AtomicU64,
}

/// Immutable snapshot of Runtime event counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MetricsSnapshot {
    pub submitted: u64,
    pub started: u64,
    pub completed: u64,
    pub failed: u64,
    pub cancellation_requested: u64,
    pub cancellation_acknowledged: u64,
}

impl RuntimeMetrics {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_submitted(&self) {
        increment(&self.submitted);
    }
    pub fn record_started(&self) {
        increment(&self.started);
    }
    pub fn record_completed(&self) {
        increment(&self.completed);
    }
    pub fn record_failed(&self) {
        increment(&self.failed);
    }
    pub fn record_cancellation_requested(&self) {
        increment(&self.cancellation_requested);
    }
    pub fn record_cancellation_acknowledged(&self) {
        increment(&self.cancellation_acknowledged);
    }

    /// Returns a thread-safe snapshot; individual counters are consistent atomic reads,
    /// but the collection is not a globally atomic point-in-time transaction.
    #[must_use]
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            submitted: load(&self.submitted),
            started: load(&self.started),
            completed: load(&self.completed),
            failed: load(&self.failed),
            cancellation_requested: load(&self.cancellation_requested),
            cancellation_acknowledged: load(&self.cancellation_acknowledged),
        }
    }
}

fn increment(counter: &AtomicU64) {
    let _ = counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value.saturating_add(1))
    });
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::RuntimeMetrics;

    #[test]
    fn records_each_event_without_conflating_cancel_request_and_acknowledgement() {
        let metrics = RuntimeMetrics::new();
        metrics.record_submitted();
        metrics.record_started();
        metrics.record_cancellation_requested();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.submitted, 1);
        assert_eq!(snapshot.started, 1);
        assert_eq!(snapshot.cancellation_requested, 1);
        assert_eq!(snapshot.cancellation_acknowledged, 0);
        metrics.record_cancellation_acknowledged();
        assert_eq!(metrics.snapshot().cancellation_acknowledged, 1);
    }

    #[test]
    fn counters_are_safe_to_increment_concurrently() {
        let metrics = std::sync::Arc::new(RuntimeMetrics::new());
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let metrics = std::sync::Arc::clone(&metrics);
                std::thread::spawn(move || {
                    for _ in 0..1000 {
                        metrics.record_completed();
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().expect("metrics thread should finish");
        }
        assert_eq!(metrics.snapshot().completed, 8000);
    }
}
