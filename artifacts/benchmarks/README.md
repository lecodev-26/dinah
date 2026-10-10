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

## SoA batch comparison — 2026-10-11

- Raw output: `memory-soa-batch-2026-10-11.txt`
- CI run: https://github.com/lecodev-26/dinah/actions/runs/38096820914
- Source commit: `1a80e013643fe4feb73e1339415070cc8160cd96`
- Runner: GitHub Actions `ubuntu-latest`, release profile
- Harness: `cargo bench --bench memory`, median of seven samples
- Comparison: `soa_push_rows` and `soa_bulk_extend` use the same precomputed input
  slices; both time buffer construction and insertion.

On this run, batch append measured 2.031 vs 6.109 ns/row at 64 rows, 0.548 vs
2.495 ns/row at 1,024 rows, and 5.837 vs 8.118 ns/row at 16,384 rows. This is
about 67%, 78%, and 28% lower measured time per row, respectively, on this runner.
These results justify keeping the batch API for this workload, but are not a
universal performance guarantee; rerun the harness on target workloads.
