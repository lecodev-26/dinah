# dinah

![dinah mark](assets/dinah-mark.svg)

**dinah** is the lowercase project name and visual identity. The initial mark is a simple, monochrome vector intended to remain legible in repository and documentation contexts.

dinah is a Rust-first project exploring a high-performance runtime and a modular foundation for compute, data-oriented processing, simulation, networking, and storage.

## Project status

**Pre-alpha / planning and foundation.** The roadmap is ambitious by design; modules are introduced only when their interfaces, tests, benchmarks, and use cases justify them.

## Development principles

- Rust is the primary implementation language.
- C, C++, and Assembly are used only where a measured technical need justifies them.
- Measure before optimizing; benchmarks must be reproducible.
- Correctness, memory safety, determinism, and clear APIs come before speculative optimization.
- Dependencies must be justified and kept maintainable.
- The roadmap describes intent; puntos.md is the numbered, living implementation checklist.
- Point IDs are permanent. Do not renumber completed, pending, or obsolete items.
- CI runs compilation, tests, and linting on GitHub Actions. Exploratory benchmarks may be run locally, but performance claims require recorded environment metadata, repeatable measurements, and CI artifacts.

## License

Unless a file states otherwise, `dinah` is dual-licensed under either the MIT License (`LICENSE-MIT`) or Apache License, Version 2.0 (`LICENSE-APACHE`), at your option. Contributions intended for inclusion are offered under the same dual-license terms unless explicitly agreed otherwise.

## Repository map

- roadmap.md: project purpose, architecture, phases, and sequencing.
- puntos.md: master checklist and development status.
- .github/workflows/ci.yml: remote continuous integration.
- src/lib.rs: initial library entry point.
- docs/runtime.md: initial Runtime execution model and deferred design decisions.
- docs/memory-model.md: Memory ownership, capacity, alignment, and synchronization design.
- docs/memory.md: practical guide to `Buffer<T>`, `SoaBuffer<A, B>`, limits, and benchmarks.
- docs/compute.md: Compute API (`for_each`, `for_each_batch`, `for_each_parallel`, `map`, `try_reduce`, `try_reduce_parallel`, `ComputePipeline`), execution/ownership/error contracts, and future implementation sequence.
- docs/ecs.md: diseño inicial del ECS, identidad de entidades con generación, almacenamiento sparse-set, contratos de World/queries y criterios de integración/benchmark.
- docs/simd-research.md: SIMD options, compatibility constraints, candidate validation matrix, and criteria before introducing vectorized kernels.

## CI

Pull requests and pushes to main run documentation/checklist validation, formatting, tests, and Clippy on GitHub-hosted runners. No local build is required.

## Current scope

The initial crate is deliberately minimal. Planned capabilities are not considered implemented until code, tests, documentation, and evidence exist.

## Core (inicial)

La API base vive en `dinah::core`; incluye configuración validada, IDs no nulos, estados de ciclo de vida y contratos compartidos de validación. Consulta [`docs/core.md`](docs/core.md) para
sus invariantes y límites actuales. El Core sigue en desarrollo y no se declara
estable hasta completar la revisión y validación previstas en el roadmap.

## Generated artifacts

- Generated benchmark artifacts are archived under `artifacts/benchmarks/<commit-sha>/run-<benchmark-run-id>/` only after both CI and the benchmark matrix succeed; see [artifact policy](artifacts/README.md).

## Automatización y bots

- [Política de automatización y bots](docs/automation-policy.md)
