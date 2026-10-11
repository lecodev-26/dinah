# dinah artifacts

This directory keeps durable, version-controlled copies of important generated
artifacts that would otherwise be available only for a limited time from CI.

## Rules

- Preserve raw output alongside a short README or metadata record describing its
  origin, commit, run URL, date, and how it was produced.
- Do not commit build directories, caches, secrets, credentials, or large
  unreviewed binary outputs. Prefer compact text, JSON, CSV, and relevant reports.
- Use descriptive, dated filenames; never silently overwrite a previous baseline.
- For benchmark results, keep the raw measurements as well as the CI run URL and
  note that timing is specific to that runner and workload.
- Keep generated artifacts distinct from source code and from the roadmap's
  completion checklist. An artifact alone does not make a roadmap point complete.

## Current contents

- `benchmarks/runtime-baseline-2026-10-10.txt` — raw 15-case output captured from
  [GitHub Actions run 38057383426](https://github.com/lecodev-26/dinah/actions/runs/38057383426),
  commit `7a3c7b0` (`bench(runtime): measure heavier CPU-bound tasks`).

- `benchmarks/memory-baseline-2026-10-10.txt` — raw memory benchmark output captured from [GitHub Actions run 38096431710](https://github.com/lecodev-26/dinah/actions/runs/38096431710), source commit `fa1ec66`. Metadata and interpretation: `benchmarks/README.md`.
- `benchmarks/memory-soa-batch-2026-10-11.txt` — raw output comparing SoA row-by-row and batch insertion from [GitHub Actions run 38096820914](https://github.com/lecodev-26/dinah/actions/runs/38096820914), source commit `1a80e01`. Summary and limits: `benchmarks/README.md`.

## Automatic archive after green CI

The `dinah artifact archive` GitHub Actions workflow archives benchmark artifacts under `artifacts/benchmarks/<commit-sha>/run-<benchmark-run-id>/`, separated by benchmark workflow run ID and then runner, with Runtime, Memory, and Compute output, environment metadata, and `archive-manifest.txt` containing the source SHA and run URL.

It writes to the repository only when the latest `dinah CI` and `dinah benchmarks` runs for the exact same `main` commit have both completed with `success`. If either run fails, is still running, or does not exist, no artifact download or commit occurs. Each benchmark workflow run has its own immutable run-ID directory, so reruns of the same source commit do not overwrite earlier measurements. Repeating the same archival event is idempotent when files have not changed.

The original Actions artifacts remain downloadable for 90 days. The version-controlled text copies preserve evidence in Git history. Do not archive build directories, secrets, credentials, or large binaries without an explicit policy.
