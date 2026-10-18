# DINAH performance baseline and benchmark-based profiling

The benchmark workflow runs seven release-mode suites: Runtime, Memory, Compute, ECS, Simulation, Network, and Storage. Each executable reports named workload cases, sample counts, the observed minimum/median/maximum sample, and normalized throughput or latency where applicable. The range is descriptive only: it is not a confidence interval, and sample counts remain modest. The runner now preserves those case lines in `baseline.json` alongside the commit, timestamp, OS, architecture, Rust toolchain, and suite wall time. To improve the scheduler comparison, the Runtime suite is executed in three independent benchmark processes by default (`DINAH_RUNTIME_BENCH_REPETITIONS`, bounded to 1–5); each run has a separate raw output file, while the original `runtime-benchmark-results.txt` and top-level `cases` field remain compatible and represent run 1. `baseline.json` stores all runs under each suite’s `repetitions` array. `baseline-summary.txt` gives a short index of suite durations, case counts, and repetition counts. CI uploads these files with the raw per-suite output.

## How to interpret the baseline

- Compare a case only with the same case, workload size, build profile, toolchain, and runner class.
- Per-case medians emitted by the benchmark executables are the primary operation-level observations; minimum and maximum expose within-process spread but do not establish statistical significance. For Runtime, compare the per-run medians across the three independent processes as well; these are repeated observations, not a confidence interval. The runs are sequential and can still be affected by thermal state, OS scheduling, and cache warmth. Suite wall time includes process startup and compilation/cache effects; it is not an operation latency.
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

The batch-vector reuse change was measured by the five-runner CI benchmark for commit `2aeee49`. It does **not** establish a consistent performance improvement. For example, on Linux x86-64 with 256 no-op tasks, the two-worker case changed from 2,539,685 ns to 2,490,303 ns and the four-worker case from 1,475,588 ns to 1,408,662 ns. On Linux ARM64 the corresponding cases moved from 2,176,193 ns to 2,286,921 ns and from 1,201,293 ns to 1,253,206 ns; on macOS ARM64 the two-worker case moved from 1,622,500 ns to 2,786,875 ns. These single-runner medians vary too much to support a general speedup claim. They are a before/after observation, not a controlled statistical study, and the synthetic no-op workload is not representative of every caller.

The benchmark also confirms that per-call worker startup and dispatch remain much more expensive than the benchmark-only persistent pool for no-op tasks. That comparison is architectural context, not a mandate to expose a persistent pool API. The next scheduler investigation removes the shared receiver mutex by assigning each scoped worker its own task channel. This is a separate implementation change; its effect must be evaluated from the CI run for that commit, and the current data does not predict the outcome.

## Scheduler channel change (commit `74ad2f0`)

The five-runner artifacts for `74ad2f0` were compared with the earlier `2aeee49` baseline for the same named scheduler workloads. Those historical commits each have a single workflow run; the new independent-process repetition is intended to strengthen future comparisons and does not retroactively add repetitions to those historical measurements. The per-worker channel implementation is mixed rather than a demonstrated general optimization: on Linux x86-64, the 256-task no-op cases moved from 2,490,303 ns to 2,572,036 ns (2 workers) and 1,408,662 ns to 2,494,662 ns (4 workers); Linux ARM64 improved the 2-worker case but regressed the 4-worker case; macOS ARM64 improved both no-op cases; macOS x86-64 and Windows x86-64 regressed substantially in these single workflow samples. CPU-heavy cases also varied by runner. Because this comparison changes code and uses one workflow run per commit, it is not sufficient to claim a reliable speedup or regression in every workload. Keep the change under review, repeat comparable runs, and do not infer allocation counts, lock contention, or cache behavior from wall-clock measurements alone.
