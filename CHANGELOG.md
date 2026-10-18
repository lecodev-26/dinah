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

### Validation

- The CI, quality/security, and documentation-audit workflows passed for commit `5762080` before the changes listed above.
- The changes in this Unreleased section require their own CI run; do not treat the prior green run as validation for them.
