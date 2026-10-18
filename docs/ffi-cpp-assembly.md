# FFI, C++, and Assembly research (points 160–163)

## C ABI design

The first ABI is deliberately scalar-only. `include/dinah.h` declares `dinah_abi_version`, `dinah_abi_features`, and `dinah_abi_platform_family`, exported by `dinah::native::ffi`. All calls use fixed-width integer values and no pointers, buffers, callbacks, Rust-owned values, or unwind paths. Consumers must check the ABI version. Feature bit 0 identifies availability of the scalar metadata query set; other bits are reserved. Platform-family values are 1 for Unix, 2 for Windows and 0 for other targets.

The crate emits `rlib`, `cdylib`, and `staticlib` artifacts so the scalar surface can be linked by Rust, C, or C++ consumers. Packaging, symbol inspection, and an actual C compiler/linker smoke test remain future validation work. The header is a contract, not proof that every target artifact has been linked and tested.

The crate-level unsafe-code lint is `deny`, with a narrow allowance only around the exported-symbol module because stable C symbol names are required. That module contains no unsafe blocks, pointer dereferences, or pointer-taking APIs. Any future pointer-based API requires a separate design for nullability, lengths, ownership, aliasing, lifetime, error mapping, panic containment, and fuzz testing before implementation.

## C++ integration

Recommended initial use is the C header from C++ via `extern "C"`, already guarded in `dinah.h`. Do not expose C++ classes, templates, exceptions, STL containers, Rust trait objects, or Rust allocation ownership across the boundary. A future C++ wrapper should be thin, RAII-oriented, and compile-tested against the public C header; it should not be introduced until a real consumer exists.

## Assembly research

No handwritten assembly is included. Rust intrinsics and compiler-generated code are preferred until profiling identifies a stable hotspot and a portable safe implementation is insufficient. Before adding assembly, require a representative benchmark, correctness reference, feature detection and fallback, ABI/register/stack review, architecture-specific CI, and a documented maintenance owner. Do not infer SIMD completion from a HAL or FFI surface.

## Explicit non-goals

These points do not provide a production hardware runtime, device discovery, direct MMIO, privileged instructions, a complete C API, C++ wrappers, or optimized assembly. See `docs/simd-research.md` for the evidence threshold for low-level optimization.
