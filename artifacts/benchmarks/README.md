# Benchmark archive

## Memory baseline — 2026-10-10

- Raw output: `memory-baseline-2026-10-10.txt`
- CI run: https://github.com/lecodev-26/dinah/actions/runs/38096431710
- Source commit: `fa1ec6623619641204c2babeb172c2a634d50004`
- Runner: GitHub Actions `ubuntu-latest`, release profile
- Harness: `cargo bench --bench memory`
- Statistic: median of seven samples per case
- Workloads: preallocated Buffer push, Buffer bulk extend, SoA row push and scan,
  and Shared mutex updates.

This is a historical baseline for the SoA batch-append comparison. It does not
include the new batch case and should not be compared across different runners
as a guarantee of performance.

## Runtime baseline — 2026-10-10

- Raw output: `runtime-baseline-2026-10-10.txt`
- CI run: https://github.com/lecodev-26/dinah/actions/runs/38057383426
- Source commit: `7a3c7b0`

Keep dated raw results immutable; add a new file for every new benchmark run.
