# DINAH — security model and review (points 195–203, 205–206, 210)

Review date: 2026-10-19. Scope: current crate surface, memory, concurrency, permissions, filesystem, networking, CLI input, Storage journal, JSON output and the scalar C ABI. Unit tests were added for Connection transitions, Storage replay, ABI regression and a bounded parallel load over independent journals. The deterministic corpus of adversarial inputs for replay and manifests was expanded; it is not presented as coverage-guided fuzzing. This static review is not an external audit and does not demonstrate the absence of vulnerabilities.

## 195 — Attack surface

- The crate declares no runtime dependencies in `[dependencies]`; the `wide` dependency is used only as a development dependency. This reduces the dependency surface but does not replace periodic lockfile/toolchain auditing.
- `dinah init` uses `create_new(true)` and does not replace an existing manifest.
- `dinah run` validates the manifest when `--task` is not given. With `--task NAME`, it explicitly executes the shell string declared for that task, in the project directory and with the user's effective permissions. There is no sandbox or process isolation; review manifests before running untrusted projects. `dinah task` only lists keys and executes nothing.
- Manifest input is limited to 1 MiB. No general protection against symbolic links, filesystem races or malicious paths on all operating systems is claimed.

## 196 — External inputs

- `DINAH.toml` is read as UTF-8; read errors are returned as I/O errors. The parser supports a deliberately reduced syntax and is not compatible with full TOML.
- A 1 MiB limit is applied before loading the whole content. Empty or unquoted project names are rejected. The current subset rejects malformed section headers, duplicated known tables, duplicate keys, assignments without `=`, and basic strings with unsupported escapes; syntax errors include the line. Tests cover these paths.
- Unknown options, unknown formats and surplus positional arguments are rejected with a non-zero exit.
- Residual risk: the basic parser does not validate all TOML rules, does not implement all TOML escape/unicode rules and does not normalize paths; it must not be used as a security validator for complex configurations. The corpus tests are deterministic and bounded; they do not replace coverage-guided fuzzing.

## 197 — Serialization

- The CLI's JSON output escapes quotes, backslashes, newlines, tabs and the remaining control characters through JSON escapes. Tests cover these cases. The parser ignores comments only outside quoted values and keeps a bounded deterministic corpus of adversarial inputs; this is not equivalent to coverage-guided fuzzing. The new metrics export produces a deterministic JSON array from the ordered snapshot and escapes arbitrary metric names.
- There is no JSON deserializer or networking wire-format decoder declared as implemented in this review. A documented format must not be assumed to already have an encoder/decoder.
- Residual risk: the metrics export does not fix a versioned compatibility schema; consumers must treat it as a local diagnostic format for this version.

## 198 — FFI

- `include/dinah.h` and `dinah::native::ffi` expose only fixed-width integer scalar queries: ABI version, flags and platform family. They accept no pointers, buffers, callbacks or transferred ownership.
- The crate keeps `#![deny(unsafe_code)]`; the FFI module only allows C symbol names and contains no unsafe blocks.
- The consumer must check `dinah_abi_version()` before using the other functions. Bits not defined in `dinah_abi_features()` are reserved.
- Integration evidence: the `quality-security.yml` workflow compiles `tests/ffi_smoke.c` with a C compiler, links it against `target/release/libdinah.a` and runs the binary; the job passed in CI on commit `fcde532`. This validates only the scalar queries present in that smoke test and does not demonstrate general ABI compatibility, coverage of all targets or release stability.

## Pending actions

- Run coverage-guided fuzzing (libFuzzer) for the manifest parser and journal replay; the current deterministic corpus is only a first regression barrier.
- Fuzzing and formal dependency auditing remain pending. The current stress/concurrency tests are bounded; they include no model checking, concurrent writers on the same journal or prolonged loads.
- Keep the C/link smoke test in CI and extend it only when new ABI functions are added; before declaring the ABI stable, review compatibility, versioning and supported targets.
- Task execution was added with explicit selection; evaluate process isolation and resource controls before considering untrusted projects. Repeat this review when introducing network deserialization, external file access or pointer-based FFI APIs.


## 199 — Memory review

- `#![deny(unsafe_code)]` is enabled at the crate level; the only `allow(unsafe_code)` is limited to the C symbol-names module, which contains no unsafe blocks or pointers.
- `Buffer<T>` and `SoaBuffer<A, B>` expose only initialized elements through Rust references. Their capacities are element limits, not byte limits or resident-memory guarantees.
- Most `Buffer` allocations are fallible; other standard containers and key/value clones may abort or enter infallible allocation paths depending on the allocator. No total resistance to memory exhaustion is claimed.
- Residual risk: there is no fuzzing or Miri in CI; invariant tests and the type system do not replace those tools.

## 200 — Concurrency review

- The mutable APIs of Storage, Connection and buffers require `&mut self`; they do not promise concurrent use of the same instance without external synchronization.
- `MetricsRegistry` uses a `Mutex`; poisoning becomes an error. `TaskQueue` is deliberately local and unsynchronized. The threaded Storage tests open independent journals, not the same file.
- The scheduler uses call-bounded threads, but a worker panic can propagate through the join points. There is no promise of operating-system task cancellation or process isolation.
- Residual risk: prolonged stress tests, loom/model checking and a policy for multiple writers simultaneously opening the same journal are missing.

## 201 — Permissions review

- DINAH does not elevate privileges, change file permissions or create privileged services. The CLI uses the process's effective permissions and returns I/O errors when the operating system denies an operation.
- The new manifest is created with `create_new(true)`, which avoids replacing an existing file; final permissions depend on the operating system and its umask.
- TCP connections may bind or connect addresses chosen by the caller. There is no built-in authentication, authorization, encryption or peer control.
- Operational recommendation: run with minimal privileges, restrict output paths and do not expose endpoints to untrusted networks without application-level controls.

## 202 — Filesystem review

- The CLI accepts a user-provided path and uses `DINAH.toml` inside that directory. The 1 MiB limit reduces excessive input, but `metadata` followed by a read does not eliminate TOCTOU races or guarantee symlink behavior on all systems.
- `DiskStore` opens the given path with read, append and create. It does not canonicalize or confine the path to a sandbox, does not coordinate instances opening the same journal and does not call `sync_all`.
- The journal is read fully into memory on open; although each field is limited to 16 MiB, the total file size has no global limit in this version. Journals must come from a trusted location.
- Residual risk: add a configurable global limit, per-platform permission/symlink tests and an explicit policy for opening an already-existing journal.

## 203 — Networking review

- The current layer is low-level blocking TCP. It adds no framing, message decoder, authentication, TLS, rate limiting, automatic reconnection or background tasks.
- Read methods may return partial data and the caller must define message limits. `set_read_timeout` and `set_write_timeout` are available but not applied by default.
- Do not treat received bytes as trusted objects or expose a listener to untrusted networks without size, time, identity and rate limits implemented by the application.

## 205 — Corruption tests

- `tests/storage.rs` checks invalid magic, a wrong v2 checksum, incomplete suffixes, a truncated checksum and v1 compatibility. Coverage is added for declared lengths above the limit and for invalid version/opcode values.
- CRC-32 serves to detect accidental corruption of complete records, not deliberate alteration. Complete corruption must fail; only the incomplete final suffix is recovered as an interrupted write.

## 206 — Stress tests

- The suite added to `src/observability.rs` stresses concurrent increments on the shared registry with several threads and verifies the aggregated result.
- It is a bounded, deterministic stress test, not a performance benchmark or a resource-exhaustion test. Prolonged tests, fuzzing and model checking remain pending.

## 210 — Security model

### Assets

Memory integrity, configuration files and journals, deterministic API behavior, process availability and trust limits between the application and external connections.

### Trust boundaries

- User files, network data and existing journal files are considered untrusted input.
- The operating system controls permissions, sockets and filesystem semantics; DINAH by itself does not prevent paths outside an allowed directory.
- Consuming applications validate payloads, define network framing and authentication, and manage Storage backups.

### Current guarantees and non-guarantees

Safe Rust APIs, field limits, JSON escaping, accidental-corruption detection and explicit errors are prioritized. No security is guaranteed against denial of service, power loss, concurrent writers to the same journal, malicious network peers, dependency vulnerabilities or operating-system errors.

### Pending controls

- Fuzzing of the manifest parser, journal replay and future network codecs.
- Dependency/toolchain auditing and periodic review of security advisories.
- Long-duration stress tests, permissions and symlinks on the supported platforms.
- A real C compile/link smoke test in CI before declaring the ABI stable.


## Additional test coverage (points 207–208, 217–219, 221–222 and 224)

- `src/network.rs` checks Open/ReadClosed/WriteClosed/Closed state transitions without creating sockets.
- `src/storage.rs` repeats replay of the same journal and validates the rejection of a DELETE with payload.
- `src/native.rs` keeps unit tests for HostPlatform, PosixPlatform (on Unix) and the scalar ABI symbols.
- `tests/robustness_regression.rs` checks repeatable ABI invariants, ordering and overwrite, and six workers on separate journal files with 128 updates each and subsequent reopening.
- These tests improve regression and bounded concurrency; they do not replace fuzzing, model checking, tests of multiple writers on the same file, or a C compiler/linker smoke test.


## Corpus robustness work (point 204, in progress)

- Storage now checks deterministic pseudo-random binary inputs of up to 512 bytes, every truncation of a valid record, per-byte mutations and truncated suffixes after concatenated records. The tests check the absence of panics, not that every input must be accepted.
- The CLI covers empty/unquoted names, duplicated fields and tables, malformed headers, invalid assignments, unquoted task descriptions, unsupported string escapes, assignments outside known sections, comments next to quoted values, the size limit, JSON escapes and a deterministic corpus of adversarial inputs. The integration tests invoke task, run, doctor and benchmark in JSON, including selected command execution, output capture and exit-code propagation. The semantics remain the documented subset, not full TOML. Command execution is opt-in but not isolated.
- Explicit limitation: these tests are a deterministic corpus and do not replace libFuzzer, guided coverage, sanitizers or long-duration sessions.

## Automated CI controls (2026-10-18)

- `.github/workflows/quality-security.yml` runs `cargo audit` over the lockfile, generates an LCOV artifact with `cargo llvm-cov` and compiles/links/runs a minimal C consumer against `libdinah.a`. The run on commit `fcde532` passed.
- `tests/ffi_smoke.c` checks the three public scalar symbols and basic ABI conditions; compilation, linking and execution passed in CI.
- The LCOV artifact from `9debb1b` was reviewed: 4,814/5,297 lines (90.88 %) and 670/763 functions (87.81 %). CI enforces an initial 85 % line threshold via `cargo llvm-cov --fail-under-lines 85`. The report contains no branch coverage; the aggregate percentage does not guarantee that critical paths are covered and does not replace a manual license review or an external security audit.


## Validated CI matrix (2026-10-18)

- The `.github/workflows/ci.yml` workflow includes an explicit matrix of Linux x86-64 and Linux ARM64. For both runners, CI confirmed formatting, library/integration tests, Clippy with warnings as errors and rustdoc generation without warnings on commit `7a5e076`.
- The quality/security workflow on `9debb1b` confirms dependency auditing, compile/link/run of the C consumer and LCOV generation/upload with the 85 % minimum line threshold. Windows/macOS remain outside the tests/Clippy/rustdoc matrix described here.
- The test matrix must not be confused with the benchmark matrix: Windows and macOS are included in benchmarks but are not covered by this tests/Clippy/rustdoc job.
