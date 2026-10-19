# Pre-publication / release checklist

Last release: v0.1.0 — 2026-10-19 (see `CHANGELOG.md`).

Release-readiness pre-checks to complete before publishing a release; the blockers and verification commands are the actual standards applied to the project, and the result must be re-checked for the release commit before tagging any future release.

## Results for v0.1.0 (release commit evidence)

All checks verified locally on the release commit (2026-10-19) and re-verified by CI on the same commit before tagging:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean |
| `cargo test --workspace --all-features` | all suites green — 293 tests total (285 unit/integration/determinism/contract/robustness + 8 doc-tests), 0 failures |
| `cargo run --example <each>` | all 8 examples executed successfully (exit 0) |
| `cargo test --workspace --doc` | green |
| Benchmark suite (7 benches) | executed successfully on the release commit |
| `cargo package --list` | inspected: sources, tests, benches, examples, docs, README, CHANGELOG, licenses; no CI artifacts, no assets |
| `cargo package` | builds the archive; the packaged crate compiles from the packaged file |
| `cargo publish --dry-run` | passes |
| MSRV | verified: fails on Rust 1.94, all tests pass on Rust 1.95 |
| Checklist validator | `python3 scripts/check_points.py` green (279 complete / 0 in progress / 8 APLAZADO) |
| CI | ci.yml, quality-security.yml, documentation_audit.yml green on the release commit |

Blocker review:

- Fuzzing and independent audits are deferred to the v0.2 cycle (`puntos2.md`, puntos 307–311) and documented as limits in `docs/security-review.md` and the README. Not blockers for 0.1.0: pre-alpha software with no consumers yet.
- MSRV is now explicit (`rust-version = "1.95"`), verified empirically on the release commit.
- Benchmarks exist and are archived for five platforms via CI (`artifacts/`), plus a full local run on the release commit.
- `cargo publish` was executed by the owner with an explicit API token after the release verification: `dinah 0.1.0` is live on crates.io (`cargo add dinah`). The dry-run, package inspection and MSRV verification all preceded the upload.

## Pre-release gates

- Re-run the full local verification suite (`cargo fmt --check`, clippy with `-D warnings`, full tests incl. doc-tests, benchmark suite) on the exact release commit; do not reuse green results from other commits.
- Inspect `cargo package --list` for unintended files; the crate must not ship CI artifacts, assets, or repository-only planning documents.
- Confirm `CHANGELOG.md` has a dated section for the version being released and that the summary matches the actual diff.
- Confirm `puntos.md`/`puntos2.md` status counters match the checklist markers (`python3 scripts/check_points.py`).
- Verify MSRV claim (`rust-version`) against the actual minimum toolchain that compiles the workspace.

## Publication process

1. Wait for CI green on the release commit (all four workflows that trigger for it).
2. `cargo publish` (owner decision; `--dry-run` already verified).
3. Tag `vX.Y.Z` and create the GitHub Release from the CI-green commit, with notes taken from the dated changelog section.
4. Update the repo description and README links if the version or links changed.
5. Post-publication: verify `cargo search dinah` / crates.io page renders correctly and the README image renders.

## Post-release verification

- Check that the tagged commit is the CI-green commit used as the release basis.
- Check that the changelog section referenced in the release notes exists and is dated.
- Check that the crate version published to crates.io matches the tag.
- Check that the README and docs still render correctly after publication.
