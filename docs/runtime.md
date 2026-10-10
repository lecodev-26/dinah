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

## Task contract (point 037)

A `Task` is the logical, one-shot unit of work accepted by the Runtime. It has
identity, observable lifecycle state, and eventually one terminal outcome. It is
not a worker, queue entry, scheduler, or executor. The same logical task must
not begin execution more than once, even if dispatch or worker operations fail.

The contract is deliberately split into stable semantics and a not-yet-selected
Rust representation:

- **Ownership:** once submission is accepted, the Runtime owns responsibility
  for either executing the task or reporting that it was cancelled/rejected
  according to the submission contract. An unaccepted task remains the caller's
  responsibility.
- **Single execution:** a task is started at most once. Retries, if ever added,
  must be explicit and must define whether they create a new task identity.
- **Lifecycle:** the state transitions in the next section are authoritative;
  a cancellation request is not itself proof that a running task stopped.
- **Outcome:** every terminal task distinguishes success, failure, and
  cancellation. Failure information should remain task-specific rather than be
  flattened into `CoreError`. A task cannot have two terminal outcomes.
- **Isolation:** task logic must execute outside scheduler/queue internal locks.
  Runtime bookkeeping must not imply that arbitrary user work is deterministic
  or side-effect-free.
- **Identity scope:** identity is assigned by the Runtime-facing API once the
  `TaskId` contract is defined; this section does not imply global uniqueness.
- **Representation not yet frozen:** closure type erasure, output/result handles,
  `Send + 'static` bounds, panic capture, and concrete error storage remain open
  until the executor and queue API are designed. These are implementation/API
  decisions, not permission to weaken the semantic guarantees above.

Acceptance criteria for point 037: the meaning, ownership boundary, one-shot
rule, lifecycle relationship, terminal outcome, and isolation requirement are
written down. The concrete Rust type and executable payload are separate follow-up
decisions; this point does not claim that a `Task` type exists in code.

## Task identity contract (point 038)

`TaskId` is a distinct Runtime-domain identifier for one logical task. It must
not be interchangeable with `CoreId` or a future `WorkerId`, even if those types
use the same numeric representation. This prevents accidentally passing an ID
from one resource domain into another API.

The initial contract is:

- **Representation:** a non-zero `u64`, wrapped in a dedicated `TaskId` newtype.
  Zero is reserved as invalid/uninitialized, matching the Core's sentinel policy.
- **Construction:** the initial type validates caller-supplied numeric values;
  it does not allocate IDs. Construction with zero returns a specific
  `TaskIdError`, rather than a generic `CoreError`.
- **Uniqueness:** the value type alone does not guarantee uniqueness. The
  component that creates/submits tasks must ensure no duplicate live task IDs
  within the relevant Runtime instance. The allocator and reuse policy are not
  defined by this value-type contract. No cross-process or global uniqueness is
  promised.
- **Stability:** a task's ID does not change during its logical lifetime. A retry
  must explicitly define whether it is the same logical task or a new task.
- **Value semantics:** equality, ordering, hashing, copying, display, and access
  to the numeric value are supported; serialization and persistence formats are
  deferred until a real storage/API requirement exists.
- **Scope:** `TaskId` identifies a task; it does not represent task state, prove
  that a task is registered, or authorize access to a task result.

The newtype intentionally mirrors `CoreId`'s simple non-zero value semantics
without aliasing it. It does not imply that `CoreId` must be used for Runtime
tasks, nor that `HasCoreId` should be implemented by `Task`.

Acceptance criteria for point 038: the domain, numeric representation, invalid
value, construction responsibility, uniqueness limits, stability, and public
value semantics are documented. This is a design decision; implementation and
tests remain separate work.

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

## Task state contract (point 039)

`TaskState` is a Runtime-specific enum representing the observed lifecycle of one
task. It is separate from Core's `LifecycleState`: a task's execution lifecycle
is not the lifecycle of a long-lived Runtime resource. The initial variants are
`Pending`, `Running`, `Completed`, `Failed`, and `Cancelled`.

The transition relation is deliberately closed and pure:

| Current state | Allowed next state | Meaning |
| --- | --- | --- |
| `Pending` | `Running` | The executor has claimed the task and execution is beginning. |
| `Pending` | `Cancelled` | The task was prevented from starting. |
| `Running` | `Completed` | The task returned successfully. |
| `Running` | `Failed` | Execution returned an error or a panic was captured under the selected panic policy. |
| `Running` | `Cancelled` | Running work observed and acknowledged cooperative cancellation. |
| `Completed`, `Failed`, `Cancelled` | none | Terminal states never transition again. |

Every transition not listed above is forbidden, including transitions to the same
state. A cancellation request is a request, not a state transition; a running
task remains `Running` until it acknowledges cancellation or reaches another
terminal outcome. State-checking methods must be pure: they do not mutate state,
provide synchronization, or claim that a transition has happened. The later
Task implementation must enforce transitions atomically wherever multiple
workers could race to claim or finish a task.

The state records lifecycle only, not the result payload or error details. Those
belong to a separate outcome/result contract. `TaskState` must be copyable,
comparable, debuggable, and hashable; no timestamps, worker IDs, retry counters,
or cancellation-request flag are embedded in this enum.

Acceptance criteria for point 039: the variants, allowed transition matrix,
terminal-state rule, cancellation semantics, separation from outcome data, and
concurrency enforcement requirement are documented. This point defines the
contract and does not claim that the enum or transition implementation exists.

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
