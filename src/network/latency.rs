//! Explicit round-trip-time estimation; no clock is read internally.
use core::time::Duration;

/// Exponentially weighted RTT estimator using integer nanoseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LatencyEstimator {
    smoothed_nanos: Option<u128>,
    samples: u64,
    minimum_nanos: Option<u128>,
    maximum_nanos: Option<u128>,
}
impl LatencyEstimator {
    pub const fn new() -> Self {
        Self {
            smoothed_nanos: None,
            samples: 0,
            minimum_nanos: None,
            maximum_nanos: None,
        }
    }
    pub const fn sample_count(&self) -> u64 {
        self.samples
    }
    pub fn smoothed_rtt(&self) -> Option<Duration> {
        self.smoothed_nanos.map(duration_from_nanos)
    }
    pub fn minimum_rtt(&self) -> Option<Duration> {
        self.minimum_nanos.map(duration_from_nanos)
    }
    pub fn maximum_rtt(&self) -> Option<Duration> {
        self.maximum_nanos.map(duration_from_nanos)
    }
    /// Adds a measured RTT. First sample initializes; later samples use 7/8 old + 1/8 new.
    pub fn record_sample(&mut self, sample: Duration) {
        let nanos = sample.as_nanos();
        self.smoothed_nanos = Some(match self.smoothed_nanos {
            None => nanos,
            Some(old) => old - old / 8 + nanos / 8,
        });
        self.minimum_nanos = Some(self.minimum_nanos.map_or(nanos, |old| old.min(nanos)));
        self.maximum_nanos = Some(self.maximum_nanos.map_or(nanos, |old| old.max(nanos)));
        self.samples = self.samples.saturating_add(1);
    }
}
fn duration_from_nanos(nanos: u128) -> Duration {
    let seconds = (nanos / 1_000_000_000).min(u64::MAX as u128) as u64;
    let subsec = (nanos % 1_000_000_000) as u32;
    Duration::new(seconds, subsec)
}
