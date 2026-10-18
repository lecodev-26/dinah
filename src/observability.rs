//! Opt-in, dependency-free observability primitives.
//!
//! Metrics and profiling are recorded only when callers explicitly report them;
//! this module does not instrument subsystem execution automatically.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::runtime::{TraceBuffer, TraceEvent, TraceRecordError};

/// Subsystem label for a caller-reported metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subsystem {
    Runtime,
    Memory,
    Compute,
    Network,
    Storage,
}

impl Subsystem {
    pub const ALL: [Self; 5] = [
        Self::Runtime,
        Self::Memory,
        Self::Compute,
        Self::Network,
        Self::Storage,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Memory => "memory",
            Self::Compute => "compute",
            Self::Network => "network",
            Self::Storage => "storage",
        }
    }
}

/// Counter and gauge values are intentionally distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricValue {
    Counter(u64),
    Gauge(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricError {
    EmptyName,
    Poisoned,
}

impl core::fmt::Display for MetricError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyName => f.write_str("metric name must not be empty"),
            Self::Poisoned => f.write_str("metrics registry lock is poisoned"),
        }
    }
}

impl std::error::Error for MetricError {}

/// Thread-safe named metrics registry. Counter increments saturate rather than wrap.
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    values: Mutex<BTreeMap<(Subsystem, String), MetricValue>>,
}

impl MetricsRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(
        &self,
        subsystem: Subsystem,
        name: &str,
        amount: u64,
    ) -> Result<u64, MetricError> {
        if name.trim().is_empty() {
            return Err(MetricError::EmptyName);
        }
        let mut values = self.values.lock().map_err(|_| MetricError::Poisoned)?;
        let key = (subsystem, name.trim().to_owned());
        let next = match values.get(&key) {
            Some(MetricValue::Counter(current)) => current.saturating_add(amount),
            Some(MetricValue::Gauge(_)) => amount,
            None => amount,
        };
        values.insert(key, MetricValue::Counter(next));
        Ok(next)
    }

    pub fn set_gauge(
        &self,
        subsystem: Subsystem,
        name: &str,
        value: i64,
    ) -> Result<(), MetricError> {
        if name.trim().is_empty() {
            return Err(MetricError::EmptyName);
        }
        self.values
            .lock()
            .map_err(|_| MetricError::Poisoned)?
            .insert(
                (subsystem, name.trim().to_owned()),
                MetricValue::Gauge(value),
            );
        Ok(())
    }

    pub fn snapshot(&self) -> Result<Vec<((Subsystem, String), MetricValue)>, MetricError> {
        self.values
            .lock()
            .map(|values| {
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), *value))
                    .collect()
            })
            .map_err(|_| MetricError::Poisoned)
    }
}

/// One completed caller-defined profiling interval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSample {
    pub name: String,
    pub elapsed: Duration,
}

/// Explicit profiler hook. The caller decides when a sample is complete.
pub trait ProfilerHook {
    fn finish(&mut self, sample: ProfileSample);
}

/// Collects bounded trace events, caller-reported subsystem metrics, and profile samples.
#[derive(Debug)]
pub struct Observability {
    trace: TraceBuffer,
    metrics: MetricsRegistry,
    profiles: Mutex<Vec<ProfileSample>>,
    profile_capacity: NonZeroUsize,
}

impl Observability {
    #[must_use]
    pub fn new(trace_capacity: NonZeroUsize, profile_capacity: NonZeroUsize) -> Self {
        Self {
            trace: TraceBuffer::new(trace_capacity),
            metrics: MetricsRegistry::new(),
            profiles: Mutex::new(Vec::with_capacity(profile_capacity.get())),
            profile_capacity,
        }
    }

    pub fn trace(&self, event: TraceEvent) -> Result<(), TraceRecordError> {
        self.trace.record(event)
    }

    pub fn trace_snapshot(&self) -> Result<Vec<TraceEvent>, TraceRecordError> {
        self.trace.snapshot()
    }

    #[must_use]
    pub const fn trace_capacity(&self) -> NonZeroUsize {
        self.trace.capacity()
    }

    #[must_use]
    pub const fn metrics(&self) -> &MetricsRegistry {
        &self.metrics
    }

    pub fn profile(&self, name: &str, operation: impl FnOnce()) -> Result<Duration, MetricError> {
        if name.trim().is_empty() {
            return Err(MetricError::EmptyName);
        }
        let started = Instant::now();
        operation();
        let sample = ProfileSample {
            name: name.trim().to_owned(),
            elapsed: started.elapsed(),
        };
        let elapsed = sample.elapsed;
        let mut profiles = self.profiles.lock().map_err(|_| MetricError::Poisoned)?;
        if profiles.len() < self.profile_capacity.get() {
            profiles.push(sample);
        }
        Ok(elapsed)
    }

    pub fn profile_with_hook(
        &self,
        name: &str,
        operation: impl FnOnce(),
        hook: &mut impl ProfilerHook,
    ) -> Result<Duration, MetricError> {
        let elapsed = self.profile(name, operation)?;
        hook.finish(ProfileSample {
            name: name.trim().to_owned(),
            elapsed,
        });
        Ok(elapsed)
    }

    pub fn profile_snapshot(&self) -> Result<Vec<ProfileSample>, MetricError> {
        self.profiles
            .lock()
            .map(|profiles| profiles.clone())
            .map_err(|_| MetricError::Poisoned)
    }

    /// Returns a compact health summary; it is not a security or performance audit.
    pub fn diagnose(&self) -> Result<DiagnosticReport, MetricError> {
        let metrics = self.metrics.snapshot()?;
        let profiles = self.profile_snapshot()?;
        let mut subsystem_metrics = BTreeMap::new();
        for subsystem in Subsystem::ALL {
            subsystem_metrics.insert(
                subsystem,
                metrics
                    .iter()
                    .filter(|((owner, _), _)| *owner == subsystem)
                    .count(),
            );
        }
        Ok(DiagnosticReport {
            metric_count: metrics.len(),
            profile_samples: profiles.len(),
            profile_capacity: self.profile_capacity.get(),
            trace_capacity: self.trace.capacity().get(),
            subsystem_metrics,
        })
    }
}

/// Snapshot of observability configuration and collected data volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticReport {
    pub metric_count: usize,
    pub profile_samples: usize,
    pub profile_capacity: usize,
    pub trace_capacity: usize,
    pub subsystem_metrics: BTreeMap<Subsystem, usize>,
}

impl ProfilerHook for Vec<ProfileSample> {
    fn finish(&mut self, sample: ProfileSample) {
        self.push(sample);
    }
}

#[cfg(test)]
mod tests {
    use super::{MetricValue, MetricsRegistry, Observability, Subsystem};
    use crate::runtime::TraceEvent;
    use std::num::NonZeroUsize;

    #[test]
    fn metrics_support_each_subsystem_and_distinguish_counter_from_gauge() {
        let metrics = MetricsRegistry::new();
        assert_eq!(
            metrics.increment(Subsystem::Runtime, "tasks.completed", 2),
            Ok(2)
        );
        assert_eq!(
            metrics.increment(Subsystem::Runtime, "tasks.completed", 3),
            Ok(5)
        );
        metrics
            .set_gauge(Subsystem::Memory, "bytes.live", 128)
            .unwrap();
        let snapshot = metrics.snapshot().unwrap();
        assert!(snapshot.contains(&(
            (Subsystem::Runtime, "tasks.completed".into()),
            MetricValue::Counter(5)
        )));
        assert!(snapshot.contains(&(
            (Subsystem::Memory, "bytes.live".into()),
            MetricValue::Gauge(128)
        )));
        for subsystem in Subsystem::ALL {
            metrics.increment(subsystem, "events", 1).unwrap();
        }
    }

    #[test]
    fn rejects_empty_metric_names() {
        assert!(MetricsRegistry::new()
            .increment(Subsystem::Compute, " ", 1)
            .is_err());
    }

    #[test]
    fn observability_records_trace_profiles_and_diagnostic_counts() {
        let observer =
            Observability::new(NonZeroUsize::new(4).unwrap(), NonZeroUsize::new(2).unwrap());
        observer.trace(TraceEvent::TaskSubmitted).unwrap();
        observer
            .metrics()
            .increment(Subsystem::Network, "messages.sent", 1)
            .unwrap();
        observer.profile("tiny-operation", || {}).unwrap();
        let report = observer.diagnose().unwrap();
        assert_eq!(report.metric_count, 1);
        assert_eq!(report.profile_samples, 1);
        assert_eq!(report.subsystem_metrics[&Subsystem::Network], 1);
        assert_eq!(
            observer.trace_snapshot().unwrap(),
            vec![TraceEvent::TaskSubmitted]
        );
    }

    #[test]
    fn profile_storage_is_bounded() {
        let observer =
            Observability::new(NonZeroUsize::new(1).unwrap(), NonZeroUsize::new(1).unwrap());
        observer.profile("first", || {}).unwrap();
        observer.profile("second", || {}).unwrap();
        assert_eq!(observer.profile_snapshot().unwrap().len(), 1);
    }
}
