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

## Shared contracts

`Validate` provides a common validation interface while retaining each type's
specific error. `HasCoreId` is the shared contract for future objects that own
or expose a `CoreId`; no resource type implements it until that type exists.
`CORE_INVARIANTS` lists the foundational assumptions that later modules must
preserve. These are descriptive contracts, not a substitute for module-level
checks or synchronization.

## Core v0.1 readiness gate

The Core is not considered stable merely because its crate version is `0.1.0` or CI is green. Before marking roadmap point 035 complete, review each item below against the implementation and CI evidence:

- [ ] Public types and methods have documented contracts, including invalid inputs and state transitions.
- [ ] Error types preserve actionable detail at the API where the error originates; conversions to `CoreError` are documented as intentionally coarse.
- [ ] Tests cover valid and invalid configuration (including precedence and numeric boundaries), non-zero IDs, ID trait behavior, every lifecycle transition, and stable error messages where callers may rely on them.
- [ ] The public API has been reviewed for unnecessary commitments, misleading names, and unused abstractions.
- [ ] `cargo fmt`, the full workspace test suite, Clippy, and warning-free public documentation build pass in GitHub Actions.
- [ ] No stability promise, release tag, or benchmark claim is made without separate review and evidence.

`HasCoreId` is currently an extension contract for future resource types; it has no concrete implementor yet. It is retained as a planned integration seam for managed resources, but must be reviewed before the Core API is declared stable rather than treated as proof of a complete resource model.

### Benchmark policy for the initial Core

The Core currently provides small validation and lifecycle primitives, not a representative workload for dinah's future runtime or compute layers. Do not publish synthetic throughput claims for these primitives. Roadmap point 033 remains in progress until there is a meaningful, reproducible operation to benchmark, with its workload, toolchain, machine/runner context, and baseline recorded. Performance-sensitive modules must add benchmarks alongside their first real implementation.
