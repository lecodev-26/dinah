# Native / Hardware foundation

## Scope and safety boundary

`dinah::native` provides `Hal` and `Platform` contracts, a safe standard-library `HostPlatform`, a Unix-family adapter, and a minimal scalar C ABI. This is platform metadata and integration scaffolding, not direct hardware access. No handwritten assembly, CPU-feature dispatch, direct syscalls, or unsafe pointer-based FFI is currently exposed.

## Performance investigation and optimization gate

Use the existing release benchmark suites as the first hotspot survey: `benches/runtime.rs`, `benches/memory.rs`, `benches/compute.rs`, `benches/ecs.rs`, `benches/simulation.rs`, `benches/network.rs`, and `benches/storage.rs`. The benchmark workflow records results on Linux x86-64, Linux ARM64, Windows x86-64, macOS x86-64 and macOS ARM64. Compare cases within the same target and run; these are workload-specific observations, not direct cross-OS performance rankings.

The Compute suite already compares scalar transforms, the public sequential and parallel APIs, and a safe `wide::f32x4` candidate across aligned and unaligned sizes. It checks result equivalence and reports medians. This is a portable vectorization candidate, not proof that a specific CPU instruction is emitted or that SIMD is faster. No architecture-specific intrinsic or handwritten assembly should be introduced until archived target-specific measurements show a repeatable improvement over the scalar baseline without unacceptable small-input regressions.

### Hotspot decision record

- **Current evidence:** existing suites cover scheduler/executor overhead, memory operations, scalar/parallel compute, ECS traversal, simulation steps, networking helpers and storage operations.
- **Decision:** do not claim a native hotspot or ship a native kernel solely from source inspection. Current evidence does not justify adding CPU-specific instructions or assembly as a default path.
- **Next optimization gate:** retain before/after outputs from the same runner class; include sample count, median, input size, target architecture, correctness equivalence and small-input behaviour. Revert or keep a change optional if its benefit is not repeatable.

## Architecture validation

The benchmark CI matrix builds and executes the benchmark suite on five OS/architecture combinations. This is useful cross-target compilation/execution evidence, but it is not a complete hardware feature matrix and does not prove every optional CPU instruction is available. Future feature-specific paths must use runtime detection or compile-time target guarantees and keep a scalar fallback.

## C ABI

`include/dinah.h` declares only fixed-width scalar query functions. Consumers can check the ABI version before using feature flags and platform-family metadata. No pointer ownership, borrowed string, allocation, callback or unwind boundary crosses the ABI. A real C compiler/linker smoke test remains outstanding.
