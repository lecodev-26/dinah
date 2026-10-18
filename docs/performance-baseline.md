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

## Initial hotspot review (commit `3a36b73`)

The archived baseline contains five runner-specific result sets. In the `scheduler_queue_parallel_noop_*` cases, the per-call scheduler took longer than the benchmark-only persistent-pool comparison for all tested task counts on each runner. For example, on the Linux x86-64 runner, 256 no-op tasks measured 2,539,685 ns with two scheduler workers and 1,475,588 ns with four, versus 50,516 ns and 54,683 ns for the already-running comparison pool. This is evidence of substantial dispatch/worker-lifecycle overhead for tiny tasks in this synthetic workload, not proof that persistent workers should be added to DINAH's public API. CPU-heavy cases behave differently, and runner values must not be compared across platforms as absolute performance rankings.

A small allocation improvement is being evaluated in `Scheduler::run_parallel`: the temporary batch vector now retains its capacity between batches instead of allocating a new vector each time. This preserves the important behavior that a whole batch is removed from the queue before any of its tasks are sent to workers. The follow-up CI benchmark run must provide the after measurements; until then, no speedup is claimed. The larger worker lifecycle gap remains a candidate for architectural work, not a change justified by one microbenchmark alone.
