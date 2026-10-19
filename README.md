<div align="center">

<p align="center">
  <img src="https://github.com/lecodev-26/dinah/raw/main/assets/logo.png" alt="DINAH logo" width="132" />&nbsp;&nbsp;&nbsp;
  <img src="https://github.com/lecodev-26/dinah/raw/main/assets/silueta_naranja_transparente.png" alt="Silueta naranja de DINAH" width="180" />
</p>

<img src="https://github.com/lecodev-26/dinah/raw/main/assets/banner.jpg" alt="DINAH: modular Rust runtime, compute and simulation foundation" width="100%">

[![CI](https://github.com/lecodev-26/dinah/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/lecodev-26/dinah/actions/workflows/ci.yml) [![Benchmarks](https://github.com/lecodev-26/dinah/actions/workflows/benchmarks.yml/badge.svg?branch=main)](https://github.com/lecodev-26/dinah/actions/workflows/benchmarks.yml) [![Documentation audit](https://github.com/lecodev-26/dinah/actions/workflows/documentation-audit.yml/badge.svg?branch=main)](https://github.com/lecodev-26/dinah/actions/workflows/documentation-audit.yml) [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-e87524.svg)](https://github.com/lecodev-26/dinah/blob/main/LICENSE-MIT)

<a href="roadmap.md"><picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/b-rel.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/b-rel.svg" alt="Roadmap" width="104"></picture></a>&nbsp;<a href="puntos.md"><picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/b-sec.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/b-sec.svg" alt="Master checklist" width="154"></picture></a>&nbsp;<a href="docs/index.md"><picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/b-docs.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/b-docs.svg" alt="Documentation" width="151"></picture></a>&nbsp;<a href="artifacts/README.md"><picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/b-arena.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/b-arena.svg" alt="Benchmark artifacts" width="171"></picture></a>

**A modular Rust foundation for runtime execution, compute, data-oriented systems and simulation.**<br>A plain-language overview of DINAH's architecture, current implementation and roadmap.

</div>

> **Version status (updated 2026-10-19):** **v0.1.0 released** — tag [`v0.1.0`](https://github.com/lecodev-26/dinah/releases/tag/v0.1.0), dated changelog in `CHANGELOG.md`, and a package verified with `cargo publish --dry-run` ready for crates.io (publication on crates.io is an explicit owner decision; the crate name is available). DINAH remains pre-alpha software: semver 0.x permits breaking changes, and the documented limitations below still apply. Core, Runtime, Memory, Compute and ECS foundations exist; Simulation's first five bounded models are implemented. Phase 7 (Networking) has completed points 120–139: API boundaries, the blocking byte-stream transport contract, logical message envelope and versioned binary wire format are documented; initial TCP connection and listening-endpoint wrappers are implemented. The wire encoder/decoder remains unimplemented; local connection lifecycle state is tracked; generic client-side prediction, bounded input history, reconciliation and replay primitives are implemented; application-level retry bookkeeping, RTT estimation, explicit liveness/backoff policy, bounded Runtime dispatch, tick-based simulation delivery, deterministic loss/latency simulation and a Networking benchmark suite are present. Full protocol/domain integration and automatic reconnection remain pending. A green CI gate validates the checks that ran; it is not a claim of API stability beyond the frozen v0.1 surface or of performance guarantees.

<div align="center">

**Repository snapshot (2026-10-19):** 287 master checklist points · 279 complete · 0 in progress · 8 deferred by design (tracked as v0.2 goals in `puntos2.md`)
**Code snapshot:** single Rust crate · 7 benchmark suites (Runtime, Memory, Compute, ECS, Simulation, Networking, Storage) · 8 runnable examples · five-platform benchmark history
**Current milestone:** v0.1.0 is released: the v0.1 public API surface and architecture are frozen (`docs/public-api-review.md`, `docs/integration-matrix.md`), the full local verification suite passed on the release commit, and the package is ready for crates.io. Native foundations include safe host/POSIX capability traits and a minimal scalar C ABI; the scalar ABI has passed a real C compiler/linker smoke test in CI; this is a limited ABI check, not broad cross-target compatibility. No direct hardware access or handwritten assembly is claimed. Networking has initial blocking TCP wrappers and prediction/reconciliation/replay primitives, but its wire encoder/decoder and full protocol integration remain pending (v0.2 cycle, `roadmap2.md`).

</div>

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-overview.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-overview.svg" alt="DINAH overview" width="100%"></picture>

</div>

DINAH is a modular systems-software project written primarily in Rust. It is building reusable foundations for task execution, memory management, compute workloads, entity-component data and deterministic simulation, with networking and other capabilities planned in staged roadmap phases.

| 🧩 Runtime | ⚙️ Compute | 🌐 Systems |
|---|---|---|
| Task scheduling, execution, synchronization and cancellation. | Sequential and parallel operations, batches, reductions and pipelines. | ECS foundations, bounded simulation models and a networking design in progress. |

DINAH is **not a finished general-purpose runtime or a game engine**. The roadmap expresses direction; the master checklist records implementation status. APIs can change while the project is pre-alpha.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-how.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-how.svg" alt="How DINAH is structured" width="100%"></picture>

</div>

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/flow.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/flow.svg" alt="DINAH module status at v0.1.0: 12 subsystems with checklist completion and deferred points" width="100%"></picture>

</div>

The design separates shared contracts from runtime execution, storage and compute concerns. Simulation builds on these foundations. Networking has initial TCP byte-stream wrappers and bounded reliability/simulation helpers, but no completed wire encoder/decoder or production network stack. Storage now has an in-memory key-value API, append-only journal with CRC-32 checksums and legacy v1 replay, ordered borrowed iteration, and an explicit bounded Runtime mutation queue; it is not a transactional database.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-principles.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-principles.svg" alt="DINAH design principles" width="100%"></picture>

</div>

1. **Correctness before optimization.** Define contracts, failure behaviour and tests before tuning performance.
2. **Measure before claiming.** Benchmark results need reproducible inputs, recorded environments and retained artifacts.
3. **Rust-first and memory-safe.** Lower-level languages are considered only when a measured technical need justifies them.
4. **Determinism where promised.** Ordering, time, state transitions and error handling must be explicit and tested.
5. **Small, justified dependency surface.** Dependencies require a concrete purpose and a maintainable upgrade path.
6. **Evidence-driven progress.** A design document is not proof of implementation; checklist points close only with appropriate code and validation evidence.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-layout.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-layout.svg" alt="DINAH repository layout" width="100%"></picture>

</div>

| Component | What it contains |
|---|---|
| `src/core` | Shared types, IDs, configuration, states, errors and core contracts. |
| `src/runtime` | Tasks, workers, scheduler/executor, synchronization, cancellation, metrics and tracing. |
| `src/memory` | Buffers, contiguous storage and structure-of-arrays foundations. |
| `src/compute` | Sequential and parallel iteration, batches, reductions and pipelines. |
| `src/ecs` | Generational entities, `World`, component storage and access contracts. |
| `src/simulation` | Fixed-step clock, deterministic step contract and bounded physics/fluid/cellular/raycast/swarm models. |
| `docs/` | Module guides, design decisions, research, benchmark notes, initial security review and engineering policies. |
| `tests/` · `benches/` · `examples/` | Tests, performance suites and runnable examples. |
| `scripts/` · `.github/workflows/` | Checklist/documentation checks, CI, benchmark matrix, artifact archiving and README status-graphic generation. |
| `artifacts/` | Retained benchmark outputs and run metadata. |

**Requirements:** Rust 1.95 or newer (verified MSRV) and Git. The crate uses Rust edition 2021 and has no runtime dependencies. Platform-specific performance measurements are collected by CI and are not universal hardware comparisons.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-crypto.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-crypto.svg" alt="DINAH modules" width="100%"></picture>

</div>

| Module | Current state | Reference |
|---|---|---|
| **Core** | Shared types and contracts exist. Core v0.1 is not declared stable; first benchmarks and API-stability decisions remain open. | [`docs/core.md`](docs/core.md) |
| **Runtime** | Scheduler, executor, task queues, parallel dispatch, synchronization, cancellation, outcomes, metrics and tracing are implemented. | [`docs/runtime.md`](docs/runtime.md) |
| **Memory** | `Buffer<T>` and `SoaBuffer<A, B>` provide initial buffer/storage abstractions; optimization remains evidence-driven. | [`docs/memory.md`](docs/memory.md) · [`docs/memory-model.md`](docs/memory-model.md) |
| **Compute** | Sequential/batched/parallel operations, reductions and `ComputePipeline` exist. SIMD remains open until a real kernel demonstrates benefit with evidence. | [`docs/compute.md`](docs/compute.md) · [`docs/simd-research.md`](docs/simd-research.md) |
| **Data / ECS** | Generational entities, `World`, sparse-set component storage and access-conflict planning exist. Full query and parallel-system execution work remains. | [`docs/ecs.md`](docs/ecs.md) · [`docs/ecs-benchmarks.md`](docs/ecs-benchmarks.md) |
| **Simulation** | Fixed-step clock, deterministic step contract, ECS bindings, replay tests and five bounded models: ASCII physics, scalar fluid diffusion, cellular automata, 2D raycasting and 2D flocking. Not a general-purpose physics or CFD engine. | [`docs/simulation.md`](docs/simulation.md) |
| **Observability** | Opt-in tracing, caller-reported counters/gauges, bounded profile samples, diagnostics and JSON metric snapshots. Not automatic subsystem instrumentation or a remote exporter. | [`docs/observability.md`](docs/observability.md) |
| **Networking** | Initial blocking TCP wrappers, explicit reliability helpers and deterministic link simulation exist. Wire encoder/decoder remains unimplemented. | [`docs/network.md`](docs/network.md) |
| **Storage** | Byte-oriented API, in-memory store, append-only journal with CRC-32 on v2 records and legacy v1 replay, ordered borrowed scans, and explicit bounded Runtime mutation dispatch. Transactions, secondary indexes, compaction and strong crash durability remain unimplemented. | [`docs/storage.md`](docs/storage.md) |

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-status.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-status.svg" alt="Implementation status" width="100%"></picture>

</div>

**Implemented foundations include:** runtime scheduling and task execution; memory buffers; sequential and parallel compute operations; generational ECS entities and sparse-set component storage; fixed-step simulation and five focused simulation models; CI, benchmark and documentation workflows.

**Not yet complete:** a production SIMD kernel, full parallel ECS system execution, full wire framing/serialization and authenticated transport, transactional storage and compaction, direct hardware integration, full CLI manifest/TOML compatibility, automatic observability, coverage-guided fuzzing and full-system application validation — all scheduled in the v0.2 cycle (`roadmap2.md`). A limited C ABI smoke test and focused integration pipelines now run in CI; neither implies broad cross-target ABI support or a production application. See the master checklist for exact status rather than treating planned work as shipped.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-bench.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-bench.svg" alt="DINAH benchmarks" width="100%"></picture>

</div>

DINAH tracks median timings for the simulation clock, simulation step, mutable entity traversal and repeated linear searches. The recorded five-platform CI matrix below is historical evidence from workflow run [`38484605568`](https://github.com/lecodev-26/dinah/actions/runs/38484605568), commit `83cbcc3`; it is not a controlled cross-OS comparison or a performance guarantee. The full suite (Runtime, Memory, Compute, ECS, Simulation, Networking, Storage) was also executed locally on the v0.1.0 release commit and archived by CI in [`artifacts/`](artifacts/README.md). Lower timings are better for these measurements.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/bars.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/bars.svg" alt="Benchmark comparison graphic" width="100%"></picture>

</div>

| Platform | Clock (ns/advance) | Step (ns/step) | Mutable walk (ns/entity) | 16 linear searches (ns/search) |
|---|---:|---:|---:|---:|
| Linux x86-64 | 7.801 | 12.547 | 0.368 | 2,437.000 |
| Linux ARM64 | 5.855 | 8.000 | 0.438 | 1,792.938 |
| Windows x86-64 | 7.836 | 12.242 | 0.346 | 2,446.062 |
| macOS x86-64 | 10.488 | 10.305 | 0.823 | 4,990.062 |
| macOS ARM64 | 2.395 | 6.012 | 0.281 | 1,533.875 |

These are medians of nine samples; construction is included in the clock and step cases. A lightweight compute kernel also showed that dispatch overhead can make parallel execution slower than sequential execution. Benchmark history and metadata are archived in [`artifacts/README.md`](artifacts/README.md). SIMD and data-oriented layout work remain open until representative workloads justify changes.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-versions.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-versions.svg" alt="DINAH roadmap phases" width="100%"></picture>

</div>

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/versions.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/versions.svg" alt="dinah v0.1.0 phase dashboard: 279 of 287 checklist points complete, 8 deferred by design" width="100%"></picture>

</div>

**Released — v0.1.0 (2026-10-19).** The 287-point v0.1 checklist is closed: 279 complete, 0 in progress, 8 deferred by design and now tracked as v0.2 goals in `puntos2.md`. The v0.1 scope ended at point 277; points 278–287 (the v0.2 design cycle) were completed as the design recorded in [`roadmap2.md`](roadmap2.md) and [`puntos2.md`](puntos2.md) (puntos 288–326). Release evidence: fmt/clippy/tests/doc-tests/benchmarks green on the release commit, `cargo package` and `cargo publish --dry-run` verified, MSRV pinned to Rust 1.95 (verified empirically), changelog dated. Networking remains a pre-alpha foundation with wire encoder/decoder and full protocol integration pending — now scheduled as Phase 18 of the v0.2 cycle.

**Deferred by design (retaken as v0.2 goals in `puntos2.md`; see `roadmap.md` for the v0.1 context):**

| v0.1 point | Area | v0.2 point in `puntos2.md` | Deferred work |
|---|---|---|---|
| 033 | Core | 313 | First benchmarks when a representative Core workload exists |
| 035 | Core | 314 | Stabilize Core v0.1 |
| 084 | Compute | 317 | SIMD only where a real benefit is demonstrated |
| 085 | Compute | 318 | Data-oriented layout optimization |
| 100 | Data / ECS | 302 | Validated and isolated parallel systems execution |
| 103 | Data / ECS | 303 | Further memory-access optimization and evidence |
| 204 | Security | 310 | Coverage-guided fuzzing with libFuzzer |
| 165 | Performance | 319 | Hotspot optimization with real profiles |

The CLI baseline and opt-in observability primitives are implemented; tracing, metrics and profiling still require explicit caller reporting and are not automatic. The v0.2 cycle (fases 18–23 in `roadmap2.md`) covers wire framing and event codec, public ECS queries, full TOML, optional sandboxing, atomic journal transactions, branch coverage, fuzzing and evidence-based optimization. Checklist counts measure tracked work items, not percentage of release effort. See [`roadmap.md`](roadmap.md), [`roadmap2.md`](roadmap2.md), the [`287-point master checklist`](puntos.md) and the [`v0.2 checklist`](puntos2.md).

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-quick.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-quick.svg" alt="DINAH quick start" width="100%"></picture>

</div>

Once published on crates.io (owner decision; the verified package is ready):

```bash
cargo add dinah
```

Or directly from the repository:

```bash
git clone https://github.com/lecodev-26/dinah.git
cd dinah
cargo build --release
```

The library API is pre-alpha and can change between minor versions while below 1.0. Start with [`docs/compute.md`](docs/compute.md) or [`docs/simulation.md`](docs/simulation.md) for implemented APIs. CI is the validation gate; benchmark claims should be checked against the corresponding run artifacts.

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-docs.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-docs.svg" alt="DINAH documentation" width="100%"></picture>

</div>

| Document | Covers |
|---|---|
| [Documentation index](docs/index.md) | Guided navigation to API guides, engineering evidence and runnable examples |
| [Core](docs/core.md) | Shared types and contracts |
| [Runtime](docs/runtime.md) | Scheduling, tasks and execution |
| [Memory](docs/memory.md) · [Memory model](docs/memory-model.md) | Buffer and storage foundations |
| [Compute](docs/compute.md) · [SIMD research](docs/simd-research.md) | Compute operations and SIMD evidence |
| [ECS](docs/ecs.md) | Entities, components and access contracts |
| [Simulation](docs/simulation.md) | Simulation models and deterministic stepping |
| [Networking](docs/network.md) | Initial API design, TCP wrappers, reliability helpers and boundaries |
| [Storage](docs/storage.md) | Initial byte key-value API, memory store and journal replay |
| [CLI](docs/cli.md) | CLI commands, starter manifest limits and output modes |
| [Observability](docs/observability.md) | Opt-in tracing, caller-reported metrics, profiling samples and JSON export |
| [Security review](docs/security-review.md) | Threat model, trust boundaries, filesystem, concurrency and remaining controls |
| [Roadmap](roadmap.md) · [Master checklist](puntos.md) | v0.1 phases, milestones and status |
| [Roadmap v0.2](roadmap2.md) · [v0.2 checklist](puntos2.md) | Design of the next cycle and its puntos 288–326 |
| [Changelog](CHANGELOG.md) · [Release checklist](docs/release-checklist.md) | v0.1.0 changes, release evidence and pre-publication gates |
| [Artifacts](artifacts/README.md) | Benchmark artifact archive |

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-assurance.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-assurance.svg" alt="Validation and engineering assurance" width="100%"></picture>

</div>

CI is used to validate code and documentation incrementally. Passing CI is evidence that the configured checks passed for a particular commit; it does not certify production readiness, API stability or a performance level.

| ✅ What the project tracks | ❌ What is not claimed |
|---|---|
| Automated CI and checklist integrity | API stability beyond the frozen v0.1 surface |
| Multi-platform benchmark evidence | Universal performance leadership |
| Focused deterministic simulation tests | General-purpose physics or CFD capability |
| Documented networking API boundaries | A completed network stack or wire protocol |
| A phased, auditable master checklist | That roadmap items are already implemented |

<div align="center">

<picture><source media="(prefers-color-scheme: dark)" srcset="https://github.com/lecodev-26/dinah/raw/main/assets/dark/h-license.svg"><img src="https://github.com/lecodev-26/dinah/raw/main/assets/light/h-license.svg" alt="License" width="100%"></picture>

</div>

Unless a file states otherwise, DINAH is dual-licensed under your choice of the [MIT License](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE). Contributions intended for inclusion follow the same dual-license terms unless explicitly agreed otherwise.
