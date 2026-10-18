# Changelog

All notable changes to DINAH are documented here. The project is pre-alpha; this file records repository changes and does not imply that a release has been published.

## Unreleased

### Added

- Explicit CLI task execution with `--task NAME`, captured JSON output, and propagated task exit status.
- Unicode escape validation for the starter manifest parser (`\\uXXXX` and `\\UXXXXXXXX`), including rejection of invalid scalar values.
- End-to-end CLI regression coverage for Unicode escapes, unknown task names, and non-zero task exit status in JSON mode.
- Initial release-readiness checklist documenting the evidence required before publishing `v0.1.0`.

### Changed

- Manifest strings preserve whitespace inside quotes, including escaped whitespace.
- CLI documentation clarifies that the manifest parser supports a starter subset rather than full TOML, and that task execution is explicit and unsandboxed.
- Public documentation now has a guided index and more precise descriptions of implemented networking, native ABI smoke-test, simulation-model, and example scope.
- The Memory example index entry now describes the bounded `Buffer<T>` behavior it actually demonstrates instead of claiming that it demonstrates `SoaBuffer`.
- README snapshot dates and the Networking benchmark checklist evidence were refreshed against the current repository state.

### Validation policy

- This changelog is a summary of repository changes, not validation evidence. CI, benchmark, and documentation-audit results must be checked against the exact commit SHA they cover; an earlier green run does not validate later changes.
- The project remains pre-alpha. This Unreleased section does not imply API freeze, package validation, legal/security approval, or release readiness.
