# Runtime benchmark archive

## Baseline — 2026-10-10

- Raw results: `runtime-baseline-2026-10-10.txt`
- CI run: https://github.com/lecodev-26/dinah/actions/runs/38057383426
- Source commit: `7a3c7b0` (`bench(runtime): measure heavier CPU-bound tasks`)
- Runner: GitHub Actions `ubuntu-latest`, release build
- Harness: `cargo bench --bench runtime`
- Statistic: median of seven samples per case
- Cases: Executor no-op, immediate scheduler, sequential queue drain, and
  sequential/parallel deterministic CPU workload with 1,000,000 integer-mixing
  iterations per task (2 and 4 workers).

These numbers are a historical, machine-specific observation, not a guarantee or
an apples-to-apples comparison with another runner. Preserve this baseline when
adding newer dated measurements so performance changes remain auditable.
