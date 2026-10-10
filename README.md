# DINAH

DINAH is a Rust-first project exploring a high-performance runtime and a modular foundation for compute, data-oriented processing, simulation, networking, and storage.

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
- All compilation, tests, linting, and benchmarks run in GitHub Actions—not on the Termux device.

## Repository map

- roadmap.md: project purpose, architecture, phases, and sequencing.
- puntos.md: master checklist and development status.
- .github/workflows/ci.yml: remote continuous integration.
- src/lib.rs: initial library entry point.

## CI

Pull requests and pushes to main run documentation/checklist validation, formatting, tests, and Clippy on GitHub-hosted runners. No local build is required.

## Current scope

The initial crate is deliberately minimal. Planned capabilities are not considered implemented until code, tests, documentation, and evidence exist.
