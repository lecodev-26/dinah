# DINAH performance baseline and benchmark-based profiling

The benchmark workflow runs seven release-mode suites: Runtime, Memory, Compute, ECS, Simulation, Network, and Storage. Each executable reports named workload cases, sample counts, and a median-based measurement. The runner now preserves those case lines in `baseline.json` alongside the commit, timestamp, OS, architecture, Rust toolchain, and suite wall time. `baseline-summary.txt` gives a short index of suite durations and case counts. CI uploads these files with the raw per-suite output.

## How to interpret the baseline

- Compare a case only with the same case, workload size, build profile, toolchain, and runner class.
- Per-case medians emitted by the benchmark executables are the operation-level observations. Suite wall time includes process startup and compilation/cache effects; it is not an operation latency.
- The five CI runners help expose architecture and operating-system differences. Do not merge their values into one universal number.
- These suites provide **benchmark-based profiling** of known workloads. They are not sampled CPU profiles, allocation profiles, cache-miss measurements, or proof of a production bottleneck. Use a platform-appropriate profiler before claiming those details.
- A named case is a candidate for investigation, not by itself proof that optimizing it improves a real application. Preserve the workload and compare before/after on the same runner.

## Current coverage

| Suite | Representative workload areas |
|---|---|
| Runtime | Executor, immediate scheduling, queues, worker startup/reuse comparisons |
| Memory | Preallocated/bulk buffer writes, SoA operations, AoS/SoA scans, shared-lock updates |
| Compute | Scalar, batch, parallel and SIMD-candidate workloads |
| ECS | World lifecycle, dense/sparse component access, stale generations, replacement |
| Simulation | Fixed-step and simulation kernels represented in `benches/simulation.rs` |
| Network | Latency estimator and bounded reliability/outbox operations |
| Storage | MemoryStore put/get and ordered scans |

## Hotspot decision gate

Roadmap item 238 remains open until comparable runner artifacts have been inspected together and candidate workloads are justified by repeatable results. Items such as allocation counts, cache locality, and sampled CPU hotspots require dedicated profiler evidence; the baseline format deliberately does not invent those measurements.
