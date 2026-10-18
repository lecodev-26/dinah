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

## Before freezing v0.1

- [ ] Agree the supported public surface module by module and identify experimental exports.
- [ ] Review every public error type, ownership/lifetime behavior, panic behavior and ordering guarantee.
- [ ] Confirm examples use only intended supported APIs and compile in CI.
- [ ] Resolve or explicitly exclude incomplete ECS queries/parallel systems and network wire framing/authentication from the v0.1 contract.
- [ ] Decide compatibility policy, deprecation process and Core API versioning policy.
- [ ] Review final generated rustdoc and package contents on the exact release candidate commit.

This review advances roadmap point 261 but deliberately leaves it open: the supported v0.1 surface and compatibility policy have not yet been approved, and architecture point 262 remains a separate review.
