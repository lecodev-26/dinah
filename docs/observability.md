# DINAH Observability — points 183–190

`dinah::observability` provides a first opt-in, dependency-free layer to record explicit signals. It installs no global collector and does not automatically instrument Runtime, Memory, Compute, Network or Storage.

## Tracing

`Observability` owns a fixed-capacity `TraceBuffer` and uses the lifecycle events already defined by Runtime. `trace` returns an error if the buffer is full or locked; it does not overwrite old events. The caller decides when to record each event. `trace_snapshot` returns the events in registration order.

## Per-subsystem metrics

`MetricsRegistry` records metric names per subsystem (`Runtime`, `Memory`, `Compute`, `Network`, `Storage`). `increment` keeps saturating counters to avoid overflow; `set_gauge` replaces an instantaneous value. `snapshot` returns an ordered, copied view. Metrics reflect only explicit application calls: the library does not by itself infer memory usage, network bytes, task durations or compute/storage operations.

Empty names and mutex poisoning produce errors. Names are local labels, not a versioned telemetry schema. No maximum label cardinality, aggregations or time windows are defined. `MetricsRegistry::export_json` exports an ordered snapshot as a JSON array with `subsystem`, `name`, `kind` and `value`; it escapes quotes, slashes and control characters. It returns a `String` so the caller decides where to write it; it does not open sockets or send data to external services.

## Profiler hooks

`profile(name, operation)` measures the elapsed time of a synchronous operation and stores one sample while capacity remains. Later samples do not displace old ones when the buffer fills. `profile_with_hook` delivers the sample to the caller-supplied `ProfilerHook`. If the operation panics, the panic propagates and no final sample is produced. This is not a CPU, allocation or stack-sampling profiler.

## Diagnostics

`diagnose` reports the number of metrics, the profiling sample count/capacity, the tracing buffer capacity and the number of metric names per subsystem. It performs no host, permission, security, connection-health or data-integrity checks.

## Example

```rust
use dinah::observability::{Observability, Subsystem};
use dinah::runtime::tracing::TraceEvent;
use std::num::NonZeroUsize;

let telemetry = Observability::new(
    NonZeroUsize::new(128).expect("nonzero capacity"),
    NonZeroUsize::new(64).expect("nonzero capacity"),
);
telemetry.trace(TraceEvent::TaskSubmitted)?;
telemetry.metrics().increment(Subsystem::Runtime, "tasks.submitted", 1)?;
let report = telemetry.diagnose()?;
assert_eq!(report.metric_count, 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The implementation is local, explicit and synchronous. The JSON export is a snapshot representation, not a remote protocol or a cross-version schema-compatibility guarantee. Build profiles are Cargo's debug/release profiles; the repository's reproducible benchmarks use `cargo bench` in release and record environment metadata, but results still depend on host and load.
