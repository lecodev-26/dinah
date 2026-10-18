# DINAH v0.1.0 release-readiness checklist

This is an evidence checklist, not a declaration that `v0.1.0` is ready. Do not publish a tag, package, or release until the blocking items below have been resolved or the release scope has been explicitly narrowed and documented.

## Release blockers

- [ ] Run all CI workflows on the exact release commit and confirm success.
- [ ] Decide whether the CLI manifest contract is the documented starter subset or full TOML. If the subset remains, make that limitation prominent in package metadata, README, and CLI docs; do not claim TOML compatibility.
- [ ] Complete public API and architecture review; document compatibility policy and freeze only the surface intentionally supported in v0.1.
- [ ] Resolve or document module dependency-cycle findings with evidence.
- [ ] Review license metadata, `LICENSE-MIT`, `LICENSE-APACHE`, third-party notices, and packaging contents.
- [ ] Check the final package contents with Cargo packaging tooling and inspect the generated archive before publication.
- [ ] Confirm the crate description, repository links, README rendering, version, and changelog match the exact artifact.
- [x] Initial line-coverage guardrail set to 85% and enforced by `cargo llvm-cov --fail-under-lines 85`; latest reviewed LCOV (`9debb1b`) reports 90.88% lines and 87.81% functions. Re-evaluate before release; branch coverage is not reported and this aggregate threshold does not replace critical-path review.
- [ ] Review remaining security limitations: no task sandbox, no full TOML parser, and no complete networking wire codec/authenticated transport.
- [ ] Ensure examples compile in CI and accurately describe implemented capabilities.
- [ ] Publish only after the release candidate commit has passed CI; then verify the registry page and GitHub release point to the same version/commit.

## Release commands (run only after blockers are resolved)

```bash
cargo package --list --locked
cargo package --locked
cargo publish --dry-run --locked
```

Inspect the package file list and generated archive. A successful dry run is necessary but not sufficient for publication. Do not run `cargo publish` until the owner explicitly decides to release the validated version.

## Current known limitations

- `DINAH.toml` parsing is a documented starter subset, not complete TOML.
- `dinah --task NAME run` executes the declared shell command with the caller's permissions; there is no sandbox.
- Networking has TCP wrappers and helper primitives, but no complete wire encoder/decoder, TLS, or built-in authentication.
- Observability is opt-in and caller-reported, not automatic instrumentation or a remote exporter.
- Passing CI does not establish API stability, production readiness, or performance guarantees.
