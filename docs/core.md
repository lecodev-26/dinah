# dinah Core (initial API)

The `dinah::core` module contains dependency-free primitives intended to be
shared by future subsystems. It does not start threads, perform I/O, or allocate
resources as a side effect.

- `CoreConfig` validates that worker and batch limits are non-zero. If both
  are zero, `CoreConfig::new` reports `ZeroWorkers` first. The type accepts any
  non-zero `usize`; it does not promise that the requested capacity is feasible
  for a particular machine. Its default is one worker and batches of 64 items.
- `CoreId` is a non-zero `u64` newtype. ID allocation and uniqueness across
  processes are deliberately out of scope; callers provide the numeric value.
- `LifecycleState` describes a small lifecycle contract. It reports permitted
  transitions but does not mutate or synchronize a resource.
- `CoreError` and `CoreResult<T>` provide shared error vocabulary for future
  operations; specific APIs should preserve detailed error types where useful.

This API is an initial internal foundation, not a stability guarantee. Changes
must be accompanied by tests and reviewed against the roadmap.

`CORE_API_VERSION` identifies the documented Core contract independently of the
crate's package version. Increment it when that contract changes incompatibly;
adding an API item does not automatically require an increment. Because dinah is
pre-alpha, the marker is informative and does not replace release notes, review,
or the Core v0.1 readiness gate below.

## Shared contracts

`Validate` provides a common validation interface while retaining each type's
specific error. Validation implementations should be deterministic and free of
side effects; successful validation does not guarantee that external conditions
remain unchanged. `HasCoreId` is the shared contract for future objects that own
or expose a `CoreId`; implementations must keep the identifier stable for the
value's lifetime. The trait does not allocate IDs, guarantee global uniqueness,
or imply runtime registration.
`CORE_INVARIANTS` lists the foundational assumptions that later modules must
preserve. These are descriptive contracts, not a substitute for module-level
checks or synchronization.

## Core v0.1 readiness gate

The Core is not considered stable merely because its crate version is `0.1.0` or CI is green. Before marking roadmap point 035 complete, review each item below against the implementation and CI evidence:

- [x] Public types and methods have documented contracts, including invalid inputs and state transitions. The lifecycle helper is explicitly a predicate, not a state mutator.
- [x] Error types preserve actionable detail at the API where the error originates; conversion from `CoreConfigError` to `CoreError` is documented and tested as intentionally coarse.
- [ ] Tests cover the complete contract, including concrete `HasCoreId` behavior. Configuration boundaries and precedence, non-zero IDs, the full lifecycle transition matrix, and stable error messages are covered; `HasCoreId` has no implementor to test yet.
- [x] Initial public API review completed: no concrete consumer currently needs `HasCoreId`, and its contract remains provisional rather than a reason to build a resource abstraction prematurely.
- [x] `cargo fmt`, the workspace library/integration test suite, Clippy, and warning-free public documentation build passed in GitHub Actions run [38208309238](https://github.com/lecodev-26/dinah/actions/runs/38208309238).
- [x] No stability promise, release tag, or unsupported benchmark claim is made; Core v0.1 remains gated on unresolved API evidence.

`HasCoreId` is currently an extension contract for future resource types; it has no concrete implementor yet. Its stability decision remains open: keep it only if the first real resource type can use the contract without awkward adapters or extra constraints. Until then, do not treat it as proof of a complete resource model.

### Audit result (2026-10-12)

The documented Core surface was checked against implementation and tests. The configuration error conversion intentionally loses field-level detail only when callers explicitly choose the coarse `CoreError`; `CoreConfig::new` and `Validate` retain `CoreConfigError`. The full 4×4 lifecycle transition matrix is tested. The principal unresolved v0.1 question is whether `HasCoreId` is useful for the first real resource type; adding a synthetic implementor just to satisfy the checklist would create false confidence, so that test remains blocked until a real consumer exists.

### Benchmark policy for the initial Core

The Core currently provides small validation and lifecycle primitives, not a representative workload for dinah's future runtime or compute layers. Do not publish synthetic throughput claims for these primitives. Roadmap point 033 remains in progress until there is a meaningful, reproducible operation to benchmark, with its workload, toolchain, machine/runner context, and baseline recorded. Performance-sensitive modules must add benchmarks alongside their first real implementation.
