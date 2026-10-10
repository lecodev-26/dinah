# dinah Core (initial API)

The `dinah::core` module contains dependency-free primitives intended to be
shared by future subsystems. It does not start threads, perform I/O, or allocate
resources as a side effect.

- `CoreConfig` validates that worker and batch limits are non-zero. Its default
  is intentionally conservative: one worker and batches of 64 items.
- `CoreId` is a non-zero `u64` newtype. ID allocation and uniqueness across
  processes are deliberately out of scope; callers provide the numeric value.
- `LifecycleState` describes a small lifecycle contract. It reports permitted
  transitions but does not mutate or synchronize a resource.
- `CoreError` and `CoreResult<T>` provide shared error vocabulary for future
  operations; specific APIs should preserve detailed error types where useful.

This API is an initial internal foundation, not a stability guarantee. Changes
must be accompanied by tests and reviewed against the roadmap.
