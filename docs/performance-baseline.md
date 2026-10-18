# DINAH performance baseline and benchmark-based profiling

The benchmark workflow runs seven release-mode suites: Runtime, Memory, Compute, ECS, Simulation, Network, and Storage. Each executable reports named workload cases, sample counts, the observed minimum/median/maximum sample, and normalized throughput or latency where applicable. The range is descriptive only: it is not a confidence interval, and sample counts remain modest. The runner now preserves those case lines in `baseline.json` alongside the commit, timestamp, OS, architecture, Rust toolchain, and suite wall time. To improve the scheduler comparison, the Runtime suite is executed in three independent benchmark processes by default (`DINAH_RUNTIME_BENCH_REPETITIONS`, bounded to 1–5); each run has a separate raw output file, while the original `runtime-benchmark-results.txt` and top-level `cases` field remain compatible and represent run 1. `baseline.json` stores all runs under each suite’s `repetitions` array and provides `runtime_repeated_case_summary`, which calculates the median of the process-level medians, the observed range across processes, and workload-specific ratios against the benchmark-only persistent-pool prototype. `baseline-summary.txt` includes the same repeated-case summary alongside suite durations and case counts. CI uploads these files with the raw per-suite output.

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


## Repeated Runtime results (commit `2d7e8b4`)

The five CI artifacts for `2d7e8b4` contain three independent Runtime processes each. The figures below are the median of the three executable-reported process medians for 256 no-op tasks; they are runner-local observations, not a cross-platform ranking.

| Runner | Scheduler, 2 workers | Pool prototype, 2 workers | Ratio | Scheduler, 4 workers | Pool prototype, 4 workers | Ratio |
|---|---:|---:|---:|---:|---:|---:|
| Linux x86-64 | 3,319,751 ns | 70,131 ns | 47.3× | 2,132,848 ns | 72,826 ns | 29.3× |
| Linux ARM64 | 1,881,861 ns | 124,072 ns | 15.2× | 1,394,053 ns | 133,087 ns | 10.5× |
| Windows x86-64 | 1,808,700 ns | 65,200 ns | 27.7× | 4,022,800 ns | 63,000 ns | 63.9× |
| macOS x86-64 | 10,704,208 ns | 150,065 ns | 71.3× | 7,439,933 ns | 213,277 ns | 34.9× |
| macOS ARM64 | 1,696,958 ns | 35,417 ns | 47.9× | 1,339,708 ns | 107,083 ns | 12.5× |

The repeated observations reinforce one narrow conclusion: creating and joining scoped workers for every call is expensive relative to this already-running comparison pool for a no-op workload. They do **not** prove that a persistent pool can be adopted safely or that it will improve a real application. The prototype has a different lifecycle and synchronization design; the scheduler also has result/error semantics and per-call worker startup that the prototype comparison does not model equivalently. CPU-heavy cases have smaller and runner-dependent ratios, so do not generalize the no-op result to all workloads.

The ranges across the three process medians are also runner-dependent. For example, the Linux x86-64 scheduler's 2-worker no-op result ranged from 3,303,386 to 3,327,896 ns, while macOS x86-64 ranged from 8,825,760 to 10,851,809 ns. Three runs are useful diagnostic evidence but remain too few to estimate a reliable confidence interval or establish a production bottleneck. Allocation, copy, and cache claims still require direct profiling.

## Lifecycle-inclusive pool comparison

The Runtime benchmark adds `persistent_pool_noop_lifecycle_{2,4}_workers` for each tested task count. In the CI artifacts for `6cdecd2`, the 256-task lifecycle-inclusive ratios (Scheduler divided by pool prototype) were 2.6×–27.1× across the five runner classes, compared with 6.8×–76.0× for the steady-state pool comparison. These are medians of three process-level medians, not confidence intervals. The lifecycle comparison still submitted all tasks before waiting, whereas `Scheduler::run_parallel` uses batches of at most one task per worker and waits at every batch boundary. That difference can itself dominate no-op timings, so the figures must not be interpreted as an isolated scheduler overhead measurement.

The next benchmark revision makes the pool prototype use the same batch size and completion barrier as the scheduler. It still has a different dispatch mechanism (shared receiver protected by a mutex), task-result/error handling, panic behavior, and result ordering, so it remains a diagnostic prototype rather than a semantically equivalent implementation. CI must collect the new measurements before drawing further conclusions; no public persistent-pool API or scheduler redesign is justified by the current ratios alone.


## Lifecycle-inclusive batch comparison (commit `4f223d7`)

The five runner artifacts for `4f223d7` were archived under `artifacts/benchmarks/4f223d71f83df3243a54f53388ca1ce5805b6e1f/`. The initial lifecycle-inclusive comparison attempted to use batches of at most one task per worker and a completion barrier. Review of the prototype implementation after the run found that this was **not actually enforced**: workers shared one receiver behind a mutex, so a fast worker could receive and execute more than one queued task in the same nominal batch before another worker received its task. Therefore these measurements do not establish an equivalent batch assignment and must not be treated as an isolated comparison of scheduler overhead.

For transparency, the observed 256-no-op-task scheduler/prototype lifecycle ratios from those artifacts were 0.828×/1.074× (Linux ARM64, 2/4 workers), 1.176×/1.259× (Linux x86-64), 0.963×/1.229× (macOS ARM64), 0.597×/1.042× (macOS x86-64), and 4.320×/5.347× (Windows x86-64). They are medians of three process-level medians, not confidence intervals. The wide runner-dependent spread, plus the discovered batch-assignment mismatch, prevents a general performance conclusion; the values are retained as historical observations only.

The next benchmark revision replaces the shared receiver with one dedicated channel per worker. Each batch sends at most one task to each selected worker and waits for every task in that batch before sending the next. This fixes the task-assignment flaw in the comparison harness. The prototype still differs from `Scheduler::run_parallel` in its queue/dispatch implementation, task return/error semantics, panic behavior, and collection/order of results. The revised measurements must be reviewed before updating conclusions. No production scheduler code or public API is changed by this experiment.

## Per-worker channel harness results (commit `735ee2c`)

CI passed for the harness correction and its five runner artifacts were inspected. The comparison now assigns at most one job to each selected worker per batch through dedicated channels and waits at each batch boundary. For 256 no-op tasks, the median-of-three-process-medians results were:

| Runner | Workers | Scheduler (ns) | Pool lifecycle (ns) | Ratio |
|---|---:|---:|---:|---:|
| Linux ARM64 | 2 | 1,925,648 | 1,936,594 | 0.994× |
| Linux ARM64 | 4 | 1,421,945 | 1,486,218 | 0.957× |
| Linux x86-64 | 2 | 2,305,416 | 2,262,502 | 1.019× |
| Linux x86-64 | 4 | 1,634,788 | 1,587,758 | 1.030× |
| Windows x86-64 | 2 | 546,000 | 592,500 | 0.922× |
| Windows x86-64 | 4 | 2,924,200 | 2,725,300 | 1.073× |
| macOS x86-64 | 2 | 9,490,300 | 9,354,686 | 1.014× |
| macOS x86-64 | 4 | 7,285,856 | 7,105,391 | 1.025× |
| macOS ARM64 | 2 | 1,292,583 | 1,360,167 | 0.950× |
| macOS ARM64 | 4 | 977,667 | 975,833 | 1.002× |

Here ratio is Scheduler divided by the pool prototype including its per-sample startup/shutdown; values below 1 mean the Scheduler sample was faster. The ratios cluster near parity on most runner/worker combinations, unlike the prior invalid batch-assignment comparison. However, this is still a synthetic no-op workload, only three process repetitions, and a benchmark-only prototype: dispatch details, result/error/panic handling and result ordering are not fully equivalent. The Windows four-worker and macOS ARM64 four-worker lifecycle ranges are especially wide, so their medians need cautious interpretation. This is evidence that the corrected harness no longer reproduces the prior large lifecycle gap for this workload, not evidence of a production scheduler optimization. No production code or public API was changed; items 238–245 remain open pending representative workloads, profiling where relevant, and a controlled before/after change to production code.

### CPU-heavy comparison on the same harness

The same five artifacts also report the existing `cpu_1m` workload (256 tasks, each performing one million deterministic wrapping-multiply/add iterations) against the persistent pool in steady state. Ratios below are Scheduler median divided by persistent-pool median across three independent process runs; below 1 favors the Scheduler sample, above 1 favors the pool sample.

| Runner | Workers | Scheduler (ns) | Persistent pool (ns) | Ratio |
|---|---:|---:|---:|---:|
| Linux ARM64 | 2 | 77,692,694 | 77,559,786 | 1.002× |
| Linux ARM64 | 4 | 63,290,142 | 70,138,769 | 0.902× |
| Linux x86-64 | 2 | 19,982,138 | 20,207,519 | 0.989× |
| Linux x86-64 | 4 | 16,256,171 | 18,104,152 | 0.898× |
| Windows x86-64 | 2 | 22,550,700 | 24,527,000 | 0.919× |
| Windows x86-64 | 4 | 13,087,900 | 11,765,100 | 1.112× |
| macOS x86-64 | 2 | 27,040,062 | 30,088,685 | 0.899× |
| macOS x86-64 | 4 | 16,268,163 | 17,877,892 | 0.910× |
| macOS ARM64 | 2 | 132,547,958 | 133,972,542 | 0.989× |
| macOS ARM64 | 4 | 132,358,125 | 130,994,250 | 1.010× |

These medians are close enough to show no consistent cross-platform winner: most cases favor the scheduler modestly, while Windows at four workers and macOS ARM64 at four workers favor the pool slightly. The workload is synthetic, the pool remains semantically different (including result/error/panic handling), and the measurements are only three process-level repetitions. This is not a controlled before/after production-code comparison and does not justify changing scheduler architecture. It does provide a CPU-heavy complement to the no-op comparison; items 238–245 remain open until representative application workloads, direct profiling/allocations/cache evidence, and a controlled production change are available.
