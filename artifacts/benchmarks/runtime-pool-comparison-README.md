# Runtime persistent-pool exploration

- Source commit: `68cfb348d78d9ac32f10a0292cbbb98b942925aa`
- Date: 2026-10-10
- Workflow runs: [38058048968](https://github.com/lecodev-26/dinah/actions/runs/38058048968) and [38058161242](https://github.com/lecodev-26/dinah/actions/runs/38058161242)
- Harness: `benches/runtime.rs`, release build, seven samples per case, median reported
- Archived outputs: `runtime-pool-comparison-run-38058048968.txt` and `runtime-pool-comparison-run-38058161242.txt`

## Method and caveats

The harness compares the current `Scheduler::run_parallel` implementation, which creates scoped threads for each batch, with a benchmark-only persistent pool prototype. The prototype starts 2 or 4 workers before timed task samples and reuses them; its startup plus shutdown time is measured separately. Jobs are submitted via a standard-library channel and workers share a mutex-protected receiver. This is an exploratory baseline, not a production implementation or a perfectly equivalent dispatch path.

The no-op case exposes dispatch and thread-creation overhead. The CPU case runs 1,000,000 deterministic integer-mixing iterations per task. Results are runner-specific and can vary between runs; use repeated medians as directional evidence, not a performance guarantee. No production scheduler code was changed based on this experiment.
