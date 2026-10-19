# Changelog

All notable changes to DINAH are documented here. Versions follow [SemVer 2.0](https://semver.org/); before 1.0.0, breaking changes bump the minor version.

## [0.1.0] — 2026-10-19

First tagged release. This is the v0.1 milestone: the supported surface is frozen, the release is verified locally, and the crate is prepared for publication to crates.io. The project remains pre-alpha software: APIs can still break within 0.x and the documented limitations below still apply.

[0.1.0]: https://github.com/lecodev-26/dinah/releases/tag/v0.1.0

### Added

- Explicit CLI task execution with `--task NAME`, captured JSON output, and propagated task exit status.
- Unicode escape validation for the starter manifest parser (`\\uXXXX` and `\\UXXXXXXXX`), including rejection of invalid scalar values.
- End-to-end CLI regression coverage for Unicode escapes, unknown task names, and non-zero task exit status in JSON mode.
- Initial release-readiness checklist documenting the evidence required before publishing `v0.1.0`.
- Roadmap for the next cycle in `roadmap2.md` and its checklist continuation in `puntos2.md` (points 288+).

### Changed

- Manifest strings preserve whitespace inside quotes, including escaped whitespace.
- CLI documentation clarifies that the manifest parser supports a starter subset rather than full TOML, and that task execution is explicit and unsandboxed.
- Public documentation now has a guided index and more precise descriptions of implemented networking, native ABI smoke-test, simulation-model, and example scope.
- The Memory example index entry now describes the bounded `Buffer<T>` behavior it actually demonstrates instead of claiming that it demonstrates `SoaBuffer`.
- README snapshot dates and the Networking benchmark checklist evidence were refreshed against the current repository state.
- Package metadata for crates.io: verified MSRV (`rust-version = "1.95"`, required by `AtomicU64::try_update` in the library; the `wide` benchmark dev-dependency would only require 1.89 and is not shipped), README, keywords, categories, and repository-only files excluded from the published archive.
- The v0.1 public API surface and architecture boundaries are frozen and documented in `docs/public-api-review.md` and `docs/integration-matrix.md`.

### Release verification (v0.1.0)

Evidence recorded on the release commit before tagging:

- `cargo fmt --all -- --check` clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
- `cargo test --workspace --all-features` and `cargo test --workspace --doc`: all suites green (185+ unit tests plus integration, determinism, contract and robustness suites).
- Full benchmark suite (Runtime, Memory, Compute, ECS, Simulation, Networking, Storage) executed successfully on the release commit.
- `cargo package --list` inspected; `cargo package` builds the archive and verifies it compiles; `cargo publish --dry-run` passes.
- MSRV verified empirically: the workspace fails to compile on Rust 1.94 and passes all tests on Rust 1.95.

### Validation policy

- This changelog is a summary of repository changes, not validation evidence. CI, benchmark, and documentation-audit results must be checked against the exact commit SHA they cover; an earlier green run does not validate later changes.
- The project remains pre-alpha. Semver within 0.x permits breaking changes; the compatibility policy is documented in `docs/public-api-review.md`.

## Unreleased

No changes recorded yet for the v0.2 cycle; new work will be documented here and moved to a dated `[0.2.0]` section at release time. See `roadmap2.md` and `puntos2.md` for the planned scope.

### Validation policy

- This changelog is a summary of repository changes, not validation evidence. CI, benchmark, and documentation-audit results must be checked against the exact commit SHA they cover; an earlier green run does not validate later changes.
