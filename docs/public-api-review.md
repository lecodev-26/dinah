# Public API review — pre-alpha inventory

Review date: 2026-10-18. This is an initial contract inventory for roadmap point 261, not an API freeze or a promise of semantic-version compatibility. The repository currently exposes a single Rust crate (`dinah`, edition 2021) with ten public subsystem modules and a root `crate_name()` constant function. Public items are reachable through module paths; there is no separate prelude or stable facade.

## Exported module surface

| Module | Public contract and current boundary | Stability / review concern |
|---|---|---|
| `core` | Shared configuration, identifiers, lifecycle predicates, validation traits and errors. | `HasCoreId` has no concrete consumer/implementor; keep provisional until justified. Core v0.1 gate remains open. |
| `runtime` | Immediate scheduler/executor, queues, scoped parallel batches, cancellation/synchronization and task outcomes. | `Scheduler::run_parallel` creates scoped workers per call; it is not a persistent runtime service. Error/panic and ordering contracts must remain explicit. |
| `memory` | Owned buffers and contiguous/SoA storage primitives. | Alignment, capacity, mutation and failure invariants need to stay tested; do not promise allocator-level performance. |
| `compute` | Sequential, batched and parallel iteration/reduction plus pipeline API. | Parallel dispatch is explicit; ordering/error propagation and borrowed-input constraints are part of the contract. SIMD is not a promised execution path. |
| `ecs` | Generational `Entity`, `World`, component storage and access-planning primitives. | Query APIs and validated isolated parallel systems are incomplete; do not freeze a general query/systems model prematurely. |
| `simulation` | Fixed logical clock, deterministic stepping contract, entity bindings and bounded sample models. | A step error does not promise rollback of arbitrary state mutation; models are bounded examples, not general physics/CFD. |
| `network` | Blocking TCP `Connection`/`Endpoint`, local lifecycle state, prediction/reconciliation/replay helpers, bounded outbox and tick-based simulator. | No integrated general wire encoder/decoder, TLS, authentication, automatic reconnection or production protocol. |
| `storage` | Byte-oriented key-value API, in-memory store, append-only journal, checksums and explicit work queue. | No transactions, compaction, multi-process coordination or strong power-loss durability guarantee. |
| `native` | Safe host metadata/capability traits and a minimal scalar C ABI. | No direct hardware access; C ABI smoke coverage is a limited contract check, not a broad foreign-language compatibility promise. |
| `observability` | Opt-in tracing, caller-reported metrics, bounded samples and JSON export. | Not automatic instrumentation, a remote exporter or a security/audit system. |
| CLI binary | Starter-subset `DINAH.toml` reader and explicit task execution. | Not full TOML. Selected shell tasks run unsandboxed with caller permissions; execution remains explicit opt-in. |

## Review findings and release gates

1. **Pre-alpha compatibility only.** README and module guides must continue to say that public APIs can change. Package version `0.1.0` alone does not mean API freeze.
2. **Freeze by intentional surface, not by exporting everything.** Before point 264, enumerate the types/functions intended for supported use, review naming/error types and document deprecations/evolution policy. Do not freeze experimental helpers merely because they are public today.
3. **Core extension traits need real consumers.** `HasCoreId` stays provisional; no synthetic implementation should be added only to make its checklist pass.
4. **Cross-module orchestration is caller-owned.** Scheduler, network delivery, simulation and storage work are composed explicitly by the caller; do not imply background execution or automatic integration.
5. **Security-sensitive CLI/network/storage boundaries are limited.** The CLI has no sandbox; networking has no authenticated framed protocol; storage has no transactional durability contract. These limitations block any broad production-safety claim.
6. **Evidence status.** CI validates formatting, tests, Clippy, rustdoc, the configured Linux architecture matrix, dependency audit, C ABI smoke check and an initial line-coverage floor. Passing these checks does not establish API stability, complete branch coverage, formal verification or performance guarantees.

## Implementation spot-check (2026-10-19)

This pass compared the root exports and representative public signatures in `src/lib.rs`, `src/runtime/scheduler.rs`, `src/compute.rs`, `src/ecs/world.rs`, `src/simulation.rs`, `src/network.rs`, `src/storage.rs` and `src/observability.rs`. It is a source-level review, not a replacement for reviewing every public item or generated rustdoc.

| Contract area | Evidence in the current implementation | Decision still required before freeze |
|---|---|---|
| Module boundary | `src/lib.rs` exports ten public modules directly and `crate_name()`; there is no separate stable facade/prelude. | Decide whether all currently reachable public items are supported, or identify experimental modules/items explicitly. |
| Nested result/error shape | `Scheduler::schedule` returns `Result<Result<T, E>, ScheduleError>` although the initial immediate scheduler currently always returns the outer `Ok`; `World::insert` and network/storage APIs expose subsystem-specific errors. | Decide whether to retain reserved outer error layers in v0.1 and document which variants are currently reachable. Avoid flattening errors without a deliberate compatibility decision. |
| Panic and partial mutation | `Scheduler::run_parallel` waits for the current batch and propagates a worker panic; `Compute` parallel callbacks propagate panics after joining active work; `Simulation::step` leaves the clock unchanged on update error but does not roll back state mutations. | Review panic guarantees and non-transactional mutation semantics for every callback-based API; do not imply rollback. |
| Ownership and borrowed data | `ComputePipeline` borrows its mutable slice; Memory and Storage expose borrowed slice iteration; ECS returns references tied to `World` borrows; `Connection` owns its `TcpStream`. | Verify lifetime/ownership documentation and example patterns for each exported API, including behavior during mutation and I/O failure. |
| Public data fields | Several public records expose fields directly, including `MetricsSnapshot`, `SimulationTime`, `InputFrame`, `AuthoritativeSnapshot`, `PendingMessage`, simulation model values and diagnostic reports. | Decide which records are intentionally open data-transfer structs versus invariant-bearing types that should hide fields before freezing. Field visibility changes are compatibility-sensitive. |
| Scope and security | TCP APIs are blocking byte streams without framing/authentication; the CLI explicitly runs shell tasks unsandboxed; Storage has no transaction or strong power-loss durability contract; observability is opt-in. | Keep these exclusions visible in docs and package metadata; no general production-safety claim follows from the smoke tests. |

### Source-level findings

- No immediate reason was found to change production behavior merely to simplify the API inventory. The review instead identifies compatibility decisions that need owner approval before the freeze.
- The `expect` in `World::insert` is guarded by a `TypeId`-to-concrete-storage invariant. Decide whether that invariant and panic boundary are acceptable for the supported contract; it is not currently surfaced as a typed error.
- A source grep and representative-file review cannot prove every public method has adequate rustdoc or exhaustively classify every panic path. The checklist below therefore remains open.

## Before freezing v0.1

- [ ] Agree the supported public surface module by module and identify experimental exports.
- [ ] Review every public error type, ownership/lifetime behavior, panic behavior and ordering guarantee.
- [ ] Confirm examples use only intended supported APIs and compile in CI.
- [ ] Resolve or explicitly exclude incomplete ECS queries/parallel systems and network wire framing/authentication from the v0.1 contract.
- [ ] Decide compatibility policy, deprecation process and Core API versioning policy.
- [ ] Review final generated rustdoc and package contents on the exact release candidate commit.

This review advances roadmap point 261 but deliberately leaves it open: the supported v0.1 surface and compatibility policy have not yet been approved, and architecture point 262 remains a separate review.

## v0.1 freeze decision (2026-10-19)

The v0.1 public API surface is frozen on the v0.1.0 release commit. Decisions recorded:

1. **Supported surface.** The ten modules inventoried above (Runtime, ECS, Memory, Compute, Network, Storage, Native FFI, observability, simulation models, and the dinah CLI with starter manifest subset) are the supported v0.1 contract. The inventory of signatures and limits in this document is the authoritative list.
2. **Compatibility policy.** SemVer 2.0 within 0.x: breaking changes bump the minor version and must be listed in the v0.2 migration guide (punto 306 of `puntos2.md`). No API is removed without a deprecation note in `CHANGELOG.md` for at least one minor release.
3. **Explicit exclusions from the v0.1 contract** (documented limitations, not promises): ECS component queries and parallel systems (metadatos + scheduler determinista only), network wire framing/auth/event codec (reliable TCP + simulator only), full TOML parsing in the CLI (starter subset only), task sandboxing (opt-in explicit execution, unsandboxed), and branch coverage/fuzzing guarantees (line coverage ≥85 % in CI only).
4. **Known sharp edges kept for v0.1** (from the review above): nested scheduler result values, panic/partial-mutation semantics on system errors, borrowed ownership patterns, public-field structs, and the ECS `expect` tied to `TypeId` — all documented here and stable until v0.2 revisits them.
5. **MSRV.** `rust-version = "1.95"`, verified empirically: the workspace fails to compile on Rust 1.94 and passes all tests on Rust 1.95 (`AtomicU64::try_update` is the binding feature).
6. The v0.2 design that may evolve this surface lives in `roadmap2.md` with its checklist in `puntos2.md` (puntos 288+).
