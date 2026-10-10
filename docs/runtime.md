# dinah Runtime — initial execution model

This document records the first Runtime design decision. It is a design contract,
not a claim that task execution, scheduling, or worker management is implemented.
The first implementation must be small enough to test deterministically before
parallel execution is introduced.

## Responsibilities

- **Task** represents one submitted unit of work and its observable lifecycle. It
  must not own scheduler policy or worker-management responsibilities.
- **Executor** performs a task that has already been selected for execution. It
  reports the outcome; it does not decide global priority or queue ordering.
- **Scheduler** chooses eligible work and coordinates dispatch. It must not run
  user work while holding an internal scheduling lock.
- **Queue** stores pending work and provides the ordering contract selected by the
  scheduler. Queue mechanics must remain an implementation detail unless a public
  queue API proves necessary.
- **Worker** is an execution resource, not a task and not a scheduling policy.
  Worker identity and lifecycle should be added only with the first concrete
  worker implementation.

## Task lifecycle

The initial conceptual states are:

1. `Pending` — accepted and eligible for dispatch, but not yet started.
2. `Running` — execution has begun.
3. `Completed` — execution returned successfully.
4. `Failed` — execution returned an error or panicked and the runtime captured it.
5. `Cancelled` — the runtime prevented execution from starting, or cooperative
   cancellation was acknowledged by the running task.

Allowed transitions for the initial model:

- `Pending -> Running`
- `Pending -> Cancelled`
- `Running -> Completed`
- `Running -> Failed`
- `Running -> Cancelled` only when cooperative cancellation has actually been
  observed; a cancellation request alone is not a completed cancellation.

Terminal states (`Completed`, `Failed`, `Cancelled`) do not transition again.
A task outcome must distinguish successful completion, failure, and cancellation;
these must not be inferred solely from a generic runtime lifecycle state.

## Initial delivery sequence

1. Define task identity, task state, and outcome contracts with exhaustive tests.
2. Implement a sequential executor first, making ordering and failure behavior
   deterministic and easy to verify.
3. Add a scheduler and bounded queue with explicit capacity and submission-error
   behavior; do not silently drop accepted tasks.
4. Introduce workers and parallel execution only after sequential semantics are
   stable. Keep user callbacks outside scheduler/queue locks.
5. Add cancellation and shutdown semantics before claiming robust parallel
   lifecycle behavior.
6. Add contention/race tests and benchmarks only when there is a representative
   workload and reproducible measurement procedure.

## Design constraints

- Keep the first Runtime implementation in safe Rust; no `unsafe`, FFI, or new
  dependency is justified by this design alone.
- Do not promise FIFO, fairness, work-stealing, or hard real-time behavior until
  the selected scheduler implements and tests that guarantee.
- Do not promise preemption. Cancellation of running work is cooperative unless
  a later design proves a safe alternative.
- A task ID identifies a task within the documented runtime scope; global or
  cross-process uniqueness is not implied without an explicit allocator contract.
- A failed task should retain a useful task-level error/panic outcome without
  forcing every detailed error into the coarse `CoreError` type.
- Shutdown must define whether pending tasks are drained or cancelled; the API
  must make the selected policy explicit rather than silently discarding work.

## Decisions deliberately deferred

- Closure representation and whether submitted tasks are one-shot `Send` jobs.
- Public versus private task handles and result retrieval API.
- Queue ordering, bounded-capacity defaults, and backpressure policy.
- Worker count defaults and resizing behavior.
- Panic policy and whether a panicked worker is replaced.
- Whether `HasCoreId` is useful for actual Runtime resource types; no Runtime type
  should implement it merely to justify the abstraction.

These decisions require the concrete Task and executor API plus tests. This
model is the boundary for point 036; the subsequent points cover implementation
of the individual types and mechanisms.
