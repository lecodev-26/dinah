# DINAH — SIMD research (point 083)

Status: research completed; no SIMD code is added and no speedup is claimed. The decision is limited to setting criteria for choosing a real operation before implementing point 084.

## Decision summary

**Keep the safe scalar route for now and let LLVM/Rust try to vectorize simple loops.** Do not yet introduce `std::simd`, architecture intrinsics, `unsafe`, special alignment or an external dependency. The current Compute API is generic and there is not yet a dedicated CPU benchmark demonstrating a concrete compute-bound operation where a manual SIMD route improves the end-to-end result.

## What SIMD means for DINAH

SIMD (single instruction, multiple data) applies one operation to several lanes of data in a single vector instruction. It can help in regular loops with simple arithmetic operations and contiguous access; it is not an automatic improvement. The cost of preparing vectors, handling the remainder of elements, performing conversions or moving data can exceed the savings, especially on small inputs.

The contiguity of a slice eases sequential access, but **by itself it guarantees no special alignment** and does not ensure the compiler generates vector instructions. Not every kernel is vectorizable either: dependencies between iterations, branches, aliasing, operations with side effects and strict ordering can prevent it or change the observable result.

## Options considered

| Option | Advantages | Costs and limits | Current decision |
|---|---|---|---|
| Safe, monomorphized scalar loop | Portable; simple; correctness reference; LLVM can vectorize recognizable patterns | Vectorization depends on the code, optimizations, target and flags; it must not be assumed | **Recommended base** |
| `std::simd` / Portable SIMD | Portable vector model and expressive types | Stable availability depends on the toolchain; do not commit the public API to an experimental surface without verifying stability | Not adopted yet |
| `std::arch` intrinsics per architecture | Direct control over specific CPU instructions | Target-specific API; some operations require `unsafe` and feature detection; increases the test matrix | Not adopted yet |
| External SIMD crate | Can provide mature abstractions and per-architecture dispatch | Dependency, compatibility policy, binary size and an additional contract to review | Not added without a measured case |
| Hand-written assembly | Maximum control on one target | Poor portability and maintenance; complex interaction with ABI, compiler and safety | Outside the current scope |

This evaluation does not claim that an alternative is always faster. The decision must change if reproducible measurements demonstrate a relevant benefit for a supported use case.

## Compatibility with the crate's current rules

- `src/lib.rs` contains `#![forbid(unsafe_code)]`; adding intrinsics that require `unsafe` is not compatible with the current policy without an explicit, separate security review.
- `Cargo.toml` declares no external dependencies. Keeping the base implementation dependency-free preserves the current maintenance surface.
- The Compute API operates on borrowed slices and offers generic functions. There is not yet a specialized arithmetic kernel whose semantics and performance profile justify a public vector API.
- Memory's contracts promise no special SIMD alignment. Do not add padding or speculative over-allocation to try to obtain it.
- A correct scalar route must be kept, even if a future implementation adds CPU-feature dispatch.

## Candidate for point 084

First evaluate a concrete numeric operation over contiguous slices, for example an element-wise transformation with clearly defined semantics and no dependencies between elements. Before writing a SIMD route:

1. Specify the exact semantics (overflow, NaN, infinities, signed zero and tolerance, if applicable).
2. Implement or keep a simple scalar reference and test it.
3. Confirm with generated-code inspection or suitable tools whether the scalar version already vectorizes on the chosen targets.
4. Compare the reference against the vector alternative in `--release` builds, including small and large inputs, lengths that are not multiples of the vector width and the tail.
5. Record CPU, operating system, `rustc -Vv`, target, flags, measurement method, number of samples and the statistic used. Repeat the measurements and compare results, not just one run.
6. Reject the specialized route if the improvement is not repeatable, does not compensate for the complexity or alters the expected numeric contract.

## Minimal correctness matrix

- Empty slice, one element and inputs smaller than the vector width.
- Lengths exactly and not exactly matching the width; the tail must be processed exactly once.
- Boundary values of the type, including overflow according to the chosen semantics.
- Results compared against the scalar reference; exact equality where the contract allows it and explicit tolerance for floating-point arithmetic.
- Builds and tests on the supported targets; a specialized route must never run if the CPU does not support the required features.
- Do not claim alignment or perform reads/writes outside the slice bounds.

## Exit criteria for point 083

The research recommends **not implementing manual SIMD yet**. Point 083 includes no vector code and no performance numbers. Point 084 must provide a concrete kernel, equivalence tests, a reproducible comparison against the scalar route and evidence that the benefit justifies the cost. If no such benefit is found, documenting it and keeping the scalar route is a valid outcome.

## Point 084 — initial kernel measurement and provisional decision

`benches/compute.rs` was added and the benchmark harness runs it in CI as `cargo bench --bench compute`. It compares a direct scalar loop, `compute::for_each` and a vector candidate based on the safe `wide::f32x4` API (a development dependency, not used by the runtime). All apply the in-place transformation `x = x * 1.0001 + 0.25`; the vector candidate processes four lanes and handles the remainder with a scalar tail. Exact equality between the two scalar routes is checked, plus numeric equivalence with an explicit relative tolerance for the vector route. Small and large sizes are measured, including sizes not divisible by four, with 11 samples per case and the median. The performance test does not make the candidate part of the public API.

### Recorded exploratory environment

- Device: Samsung SM-A165F; Android 16 / Termux.
- Architecture/target: `aarch64-linux-android`; ARMv8 CPU, `CPU part 0xd05`, with `asimd` advertised by the system.
- Compiler: `rustc 1.99.0 (b940084d7 2026-09-28)`, LLVM 21.1.8.
- Command: `cargo bench --bench compute`; release profile; three harness runs, each with 11 samples per case.
- Times fluctuate visibly between runs, so these are observations from this device, not portable results or a speedup claim.

### Observed results (median per run)

| Elements | Direct loop (ns) | `compute::for_each` (ns) | Prudent reading |
|---:|---:|---|---|
| 64 | 77, 77, 77 | 77, 77, 77 | No measurable difference at the observed resolution |
| 1,024 | 308, 385, 538 | 308, 385, 538 | Identical across the three runs |
| 16,384 | 3,693, 5,461, 5,308 | 3,693, 5,538, 5,154 | Small differences with variable direction |
| 262,144 | 65,154, 92,923, 80,000 | 68,615, 93,462, 81,846 | The difference shifts between roughly +5.3 %, +0.6 % and +2.3 % of time for `for_each`; it proves no SIMD improvement |

Release assembly inspection confirms vector load/store instructions in parts of the binary, but also scalar arithmetic operations in the inspected kernel. That preliminary inspection is not enough to attribute vectorization to a specific function or to claim the compiler already vectorizes the kernel end to end.

### Safe SIMD candidate and first local measurement

To test a real alternative without relaxing `#![forbid(unsafe_code)]` in DINAH, `wide = 1.7.1` was added as a development dependency only. `wide::f32x4` provides safe vector operations; the runtime and public API do not depend on it. The harness now includes lengths that are not multiples of four to exercise the scalar tail.

First local measurement with the stabilized harness (Samsung SM-A165F, Termux, AArch64; release profile; median of 11 samples, each sample measures 10 transformations and reports time per transformation):

| Elements | Direct loop (ns/op) | `compute::for_each` (ns/op) | `wide::f32x4` candidate (ns/op) | Prudent reading |
|---:|---:|---:|---:|---|
| 64 | 30 | 30 | 30 | No difference at this resolution |
| 1,024 | 246 | 246 | 246 | No measurable difference |
| 1,027 | 215 | 215 | 246 | The 3-element tail does not improve |
| 16,384 | 3,161 | 3,169 | 3,761 | Vector ~19 % slower than the direct scalar loop |
| 16,387 | 3,146 | 3,176 | 3,776 | Vector ~20 % slower; includes tail |
| 262,144 | 59,607 | 68,492 | 68,092 | Vector ~14 % slower than the direct scalar loop |
| 262,147 | 62,084 | 60,800 | 68,300 | Vector ~10 % slower; includes tail |

These values are an exploratory sample from one device and are not a cross-platform conclusion. The CI matrix then ran the same candidate on the five runners and repeated it a second time on a documentation/configuration commit with no kernel changes. Comparison of the `wide::f32x4` candidate against the direct scalar loop at two large sizes; percentages are time variation per runner, not averages across systems (negative = the candidate was faster):

| Runner | 16,384 elements: run 1 / 2 | 262,144 elements: run 1 / 2 | Reading |
|---|---:|---:|---|
| Linux x86-64 | −6.5 % / −15.2 % | −5.9 % / −11.1 % | Advantage in both runs |
| Linux ARM64 | +16.3 % / +15.9 % | +12.7 % / +14.0 % | Slower in both runs |
| macOS ARM64 | +68.7 % / +65.2 % | +25.9 % / +21.0 % | Clear regression in both runs |
| macOS x86-64 | −3.1 % / −2.5 % | −0.7 % / −0.9 % | Small advantage, possibly within noise |
| Windows x86-64 | −4.4 % / −6.8 % | +7.8 % / −5.9 % | Result not repeatable for the large size |

The vector route has a repeatable advantage on the Linux x86-64 runner, but the same implementation is worse on ARM64 and macOS ARM64, and Windows does not repeat the sign for the larger size. Virtualized runners are also not enough to decide performance for all users' machines. Both complete matrices are kept by SHA and run ID so these numbers can be reviewed and compared in the future.

### Provisional decision

Do not add SIMD to the runtime or introduce `wide` as a production dependency: DINAH still has no real production numeric kernel that justifies an architecture-specialized route, and the candidate has no portable advantage. The evidence does leave a concrete hypothesis for a future `f32` kernel specialized on x86-64, but before that the use case must be defined, measurements repeated on controlled hardware and the numeric semantics tested. Point 084 remains **in progress**; it is not declared complete merely for having a benchmark candidate that is faster on one runner.

## Cross-platform benchmark matrix

The benchmark CI uses a matrix independent of the correctness CI and runs the three harnesses in release mode on Linux x86-64, Linux ARM64, Windows x86-64 and macOS ARM64 runners. Each runner produces separate artifacts with the Runtime, Memory and Compute results, plus `environment.txt` with the system, architecture, Rust/Cargo version, SHA and command executed. The matrix continues on the remaining runners if one fails (`fail-fast: false`).

Values must be compared **within each environment** before comparing across systems: virtualized runners, frequency/power, compiler, target and machine noise can vary. Nanoseconds must not be averaged across architectures, and a Linux/Windows difference must not be treated as exclusively an operating-system effect. The current matrix covers the three requested systems and two CPU architectures (x86-64 and ARM64), but not every OS/architecture combination. If an ARM runner becomes unavailable on the plan/repository, CI will show it as an infrastructure failure; a result from another architecture must not be labeled as ARM64.
