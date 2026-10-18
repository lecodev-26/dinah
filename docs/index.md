# DINAH documentation index

DINAH is pre-alpha. These guides describe the current implementation and its limits; they do not promise a stable v0.1 API or production readiness. Use [`puntos.md`](../puntos.md) for the authoritative work-item status and [`roadmap.md`](../roadmap.md) for project direction.

## Start here

- [Repository overview](../README.md) — current capabilities, architecture, build requirements and project status.
- [Public API review](public-api-review.md) — exported surface, compatibility risks and gates before an API freeze.
- [Integration matrix](integration-matrix.md) — tested subsystem paths and what those tests do not establish.
- [Release-readiness checklist](release-checklist.md) — blockers and checks required before a v0.1.0 release. It is not a release authorization.

## API and subsystem guides

| Area | Guide | Scope note |
|---|---|---|
| Core | [Core](core.md) | Shared types and contracts; API stability remains open. |
| Runtime | [Runtime](runtime.md) | Tasks, queues, scheduler, cancellation, outcomes, metrics and tracing. |
| Memory | [Memory guide](memory.md) · [Memory model](memory-model.md) | Buffers and SoA storage; does not promise custom allocators or general-purpose pools. |
| Compute | [Compute](compute.md) | Sequential, batch, parallel, reduction and pipeline APIs. |
| Data / ECS | [ECS](ecs.md) · [ECS benchmarks](ecs-benchmarks.md) | Entities, World and component storage; full public queries and parallel systems remain incomplete. |
| Simulation | [Simulation](simulation.md) | Fixed-step execution and bounded simulation models, not a general physics engine. |
| Networking | [Networking](network.md) · [Client-side prediction research](client-side-prediction.md) | Initial TCP wrappers and helper primitives; wire encoder/decoder, TLS and built-in authentication are not implemented. |
| Storage | [Storage](storage.md) | Memory store and append-only journal; no transactions, compaction or strong crash-durability guarantee. |
| Native | [Native / hardware foundation](native.md) · [FFI, C++ and Assembly research](ffi-cpp-assembly.md) | Host capabilities and a limited scalar C ABI; no direct hardware access or handwritten assembly is claimed. |
| Observability | [Observability](observability.md) | Opt-in, caller-reported telemetry; not automatic instrumentation or a remote exporter. |
| CLI | [CLI](cli.md) | Manifest parsing is a starter subset, not full TOML; selected shell commands run without a sandbox. |

## Engineering evidence and policies

- [Performance baseline](performance-baseline.md) — benchmark-based observations and their interpretation limits; synthetic results are not proof of production hotspots.
- [SIMD research](simd-research.md) and [data-oriented research](data-oriented-research.md) — exploratory evidence and deferred optimization decisions.
- [Security review](security-review.md) — threat model, implemented checks and remaining limitations; not an external security audit.
- [Automation policy](automation-policy.md) — repository automation and artifact-retention rules.

## Runnable examples

Run an example from the repository root with `cargo run --example NAME`. CI compiles project targets; the command below is provided for local exploration and is not a claim that every example is a complete application.

| Example | Command | Demonstrates |
|---|---|---|
| Compute | `cargo run --example compute_basics` | Basic compute operations. |
| Memory | `cargo run --example memory_basics` | Bounded `Buffer` capacity, rejected insertion and reuse after `clear`. |
| Runtime | `cargo run --example runtime_basics` | Task execution and scheduling. |
| ECS | `cargo run --example ecs_basics` | Entities, World and components. |
| Simulation | `cargo run --example simulation_basics` | Fixed-step simulation. |
| Networking | `cargo run --example network_basics` | TCP loopback with explicit connection lifecycle. |
| Storage | `cargo run --example storage_basics` | Memory/disk stores and journal replay. |
| System integration | `cargo run --example system_integration` | A bounded integration path; not a production application or a complete network protocol. |

## Benchmark artifacts

The benchmark workflow retains runner-specific outputs under [`artifacts/`](../artifacts/README.md). Compare like-for-like workloads on the same runner class and inspect the recorded toolchain and run metadata. Median/minimum/maximum observations are descriptive, not confidence intervals or universal performance guarantees.
