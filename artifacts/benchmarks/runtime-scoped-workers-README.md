# Scoped worker reuse optimization

- Source commit: `4805f9604fdfdfc5badf4b9a0506c9c638989858`
- Date: 2026-10-10
- Benchmark runs: [38058454819](https://github.com/lecodev-26/dinah/actions/runs/38058454819) and [repeat run 38058504887](https://github.com/lecodev-26/dinah/actions/runs/38058504887)
- Harness: `benches/runtime.rs`, release build, seven samples per case, median reported
- Archived outputs: `runtime-scoped-workers-run-38058454819.txt` and `runtime-scoped-workers-run-38058504887.txt`
- Regression CI: [run 38058454779](https://github.com/lecodev-26/dinah/actions/runs/38058454779)

## Change

`Scheduler::run_parallel` now creates a bounded set of scoped workers once per invocation and reuses them across all batches in that call. This avoids the much larger API/lifecycle change of a background pool that persists across calls, preserves support for closures borrowing local data, and keeps the `Scheduler` value stateless. The workers are shut down before normal return or propagated task panic.

## Comparison at 256 tasks

The two pre-change runs (commits before `4805f96`) had median CPU workload times of 31.69 ms with 2 workers and 35.83 ms with 4 temporary workers per batch. The two post-change runs had medians of 24.91 ms with 2 scoped workers and 22.65 ms with 4 scoped workers, an improvement of about 21% and 37% respectively on this runner. For no-op tasks, the median fell from 12.20 ms to 3.27 ms (2 workers) and from 11.00 ms to 2.08 ms (4 workers).

These are machine-specific microbenchmark results, not guarantees. The 2-worker CPU result varied noticeably between post-change runs, but both runs favored worker reuse versus the previous implementation. Raw outputs retain all measured cases and medians for review.
