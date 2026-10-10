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

## Worker contract (point 040)

A `Worker` is an execution resource responsible for running work that an executor or scheduler has dispatched to it. It is not a `Task`, does not choose global scheduling policy, and is not itself the queue. This is a role contract; no worker type, thread pool, or background thread is implemented by documenting it.

The initial responsibilities and boundaries are:

- **Execution boundary:** accept or claim work only through the executor's defined dispatch contract, run the selected task at most once, and make its execution outcome available to Runtime bookkeeping. A worker must not silently discard work that the Runtime has already accepted.
- **Isolation:** execute user task logic outside internal scheduler and queue locks. Internal locks may protect dispatch bookkeeping, but must not be held while arbitrary user code runs.
- **Identity:** a worker's identity belongs to the worker domain and must not be represented by `TaskId` or `CoreId`. A dedicated `WorkerId` contract is the next design point (041); this section does not imply that such a type exists.
- **Lifecycle:** a worker is a Runtime execution resource, distinct from a task's `TaskState` and Core's `LifecycleState`. Concrete states and legal transitions are deferred until the worker implementation and shutdown protocol are designed.
- **Failure and shutdown:** panic handling, worker replacement, draining or cancelling dispatched/pending work, and worker termination guarantees remain explicit open decisions. No failure-recovery or shutdown guarantee is claimed before it is implemented and tested.
- **Configuration:** `CoreConfig::max_workers` is Core configuration data; it does not establish that a worker pool exists, that this value is currently consumed by Runtime, or that dynamic resizing is supported.

The first worker implementation should remain in safe Rust and keep task execution outside scheduler/queue critical sections. Thread creation, synchronization primitives, worker count semantics, and dispatch ownership must be introduced in implementation work with tests rather than inferred from this contract.

Acceptance criteria for point 040: the worker's role, separation from tasks and scheduling policy, execution/locking boundary, identity domain, and deferred shutdown/failure semantics are documented. This point defines the contract only; it does not claim a concrete `Worker` type or worker threads exist in code.

## Worker identity contract (point 041)

`WorkerId` is a Runtime-domain identifier for one worker resource. It is a
different type from `TaskId` and `CoreId`, even if all three use the same
numeric representation. The type distinction prevents passing a task identifier
where a worker identifier is expected and keeps Runtime resource identity out of
Core's more general identity domain.

The initial contract is:

- **Representation:** a dedicated `WorkerId` newtype wrapping a non-zero `u64`.
  Zero is invalid/reserved, consistent with the existing `CoreId` and planned
  `TaskId` value contracts.
- **Construction:** the value type validates a caller-supplied numeric value and
  returns a specific `WorkerIdError` for zero. It does not allocate IDs or
  register workers as a side effect.
- **Uniqueness:** the newtype alone does not guarantee uniqueness. The component
  creating/owning workers must avoid duplicate IDs among live workers in the
  relevant Runtime instance. No process-wide, cross-process, or global
  uniqueness is promised. Allocation, reuse, and persistence policies remain
  deferred until required by a concrete worker manager.
- **Stability and scope:** a worker's ID remains stable for that worker resource's
  lifetime. It identifies neither a task nor a worker's lifecycle state, thread
  handle, liveness, permissions, or task result. Replacing a failed worker must
  explicitly define whether the replacement receives a new identity.
- **Value semantics:** support equality, ordering, hashing, copying, display, and
  access to the numeric value. Serialization and persistence formats are not
  specified without a real API or storage requirement.
- **Separation from registration:** constructing a valid `WorkerId` does not prove
  that a worker exists, is running, is registered, or is owned by a particular
  Runtime instance. Those facts belong to the future worker-management API.

The intended implementation is a small safe-Rust value type with exhaustive tests
for zero rejection, valid boundary values, display, equality/order, and hashing.
This is a contract only: no `WorkerId` or `WorkerIdError` type is claimed to exist
in code until a separate implementation point adds it.

Acceptance criteria for point 041: the domain, newtype representation, invalid
value behavior, construction and allocation boundary, uniqueness limits,
stability, and value semantics are documented.

## Bounded task queue (point 044)

The first queue implementation is a generic, single-owner `TaskQueue<T>` backed
by `VecDeque<T>` and configured with a non-zero capacity. It provides FIFO
`try_push`/`pop`, length/empty/full inspection, and exposes its configured
capacity. It is deliberately not a concurrent queue and does not execute tasks.

When the queue is full, `try_push` returns `QueuePushError::Full(item)` with the
original item, preserving caller ownership and preventing silent work loss.
The queue is not yet wired into `Scheduler`; queue-backed dispatch and its
acceptance/rejection semantics belong to later Runtime points. Tests cover empty
state, FIFO order, full-queue rejection without item loss, and reuse after a pop.

## Queue-backed dispatch (point 045)

`Scheduler::dispatch_next` integrates the bounded queue and synchronous executor:
it removes at most one oldest closure from `TaskQueue`, then executes it after
the queue borrow has ended. An empty queue returns `None`; a task result or task
error is returned as `Some(result)` without erasing the task error. This is a
sequential dispatch primitive, not a worker pool, background queue consumer, or
concurrent scheduler. Queue-full rejection remains the responsibility of the
caller when inserting work.

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


## Sequential queue drain (point 046)

`Scheduler::run_until_empty` repeatedly dispatches the oldest queued closure
until no work remains, returning each task result in FIFO order. Task-level
errors are retained in the returned vector and do not prevent later queued
tasks from executing. An empty queue returns an empty vector. This is a
synchronous drain on the caller's thread; it does not imply parallelism,
background progress, panic recovery, or a persistent scheduler lifecycle.


## Bounded parallel execution (point 047)

`Scheduler::run_parallel` executes queued closures in batches of at most the
caller-supplied non-zero worker limit. Each batch is removed from the queue
before its tasks run on scoped threads. Results are returned in FIFO dispatch
order, not completion order; task-level errors are retained and later batches
continue. The queue is empty after a normal return. This initial implementation
creates temporary threads per batch rather than maintaining a worker pool. Tasks
and result/error values must satisfy `Send`; panics are not caught and propagate
to the caller. Persistent workers, cancellation, shutdown coordination, and
recovery from panicking tasks remain out of scope for this point.


## Shared-state synchronization (point 048)

`Shared<T>` is an opt-in, cloneable owner around `Arc<Mutex<T>>`. Clones point
to the same value, and `lock()` gives exclusive access through a guard. It
returns `SynchronizationError::Poisoned` if a previous thread panicked while
holding the mutex; the wrapper does not guess whether the protected value can
be safely recovered. No extra dependency or unsafe code is needed.

The lock protects only access to the wrapped value; it does not make arbitrary
operations atomic unless callers perform them while holding the guard. Keep
critical sections short and do not execute arbitrary task callbacks while holding
a guard. This primitive does not yet make `TaskQueue` or `Scheduler` themselves
concurrently mutable, and it does not promise fairness, lock-free progress, task
cancellation, or panic recovery. Tests cover shared clone visibility, contended
increments across threads, and explicit poison reporting.


## Task error classification (point 049)

`TaskFailure<E>` wraps the original task error and labels it as
`TaskFailureKind::ReturnedError`. It preserves the concrete error type and
provides borrowing, consuming, display, and error-mapping operations without
flattening details into `CoreError`. Mapping an error retains the classification.

This is deliberately a task-error value, not a scheduler rejection, lock error,
cancellation signal, or panic wrapper. The executor and scheduler continue to
return their generic `Result<T, E>` values unchanged; callers can opt into
`TaskFailure::returned(error)` when they need a stable task-level category.
Panics still unwind and are not caught or mislabeled as returned errors. This
point does not yet implement a task registry or aggregate multiple task outcomes.
Tests verify preservation of the original error, classification, display, and
error mapping.


## Cooperative cancellation token (point 050)

`CancellationToken` is a cloneable, dependency-free signal backed by an atomic
boolean. Any clone can request cancellation, and all clones can observe the
request. Repeated requests are idempotent; `request_cancellation()` returns
`true` only to the caller that first changed the flag. Acquire/release ordering
provides visibility for the cancellation flag itself, not synchronization for
unrelated task data.

Cancellation is cooperative: this token cannot interrupt a closure, kill a
thread, roll back side effects, or guarantee that work has stopped. Task code
must poll at safe points and return or otherwise complete its own cleanup. A
request must not be confused with a confirmed `Cancelled` terminal task state;
that requires task lifecycle bookkeeping and observation of the task's actual
completion, which are not implemented by this primitive. Tests cover shared
visibility across clones and idempotent concurrent requests.


## Task finalization state transitions (point 051)

`TaskState` implements the documented task lifecycle with a checked transition
method. Only `Pending -> Running`, `Pending -> Cancelled`, and `Running ->
Completed`, `Failed`, or `Cancelled` are permitted. Self-transitions and all
transitions out of terminal states are rejected. `InvalidTaskTransition` keeps
the source and requested destination for diagnostics, and rejected transitions
leave the state unchanged. `is_terminal()` identifies `Completed`, `Failed`,
and `Cancelled`.

This value type is not synchronized; callers sharing state across threads must
provide synchronization and serialize updates. It does not provide atomic task
claiming across workers. A transition to `Cancelled` from `Running` is valid
only after cooperative cancellation is actually acknowledged, not merely
requested. Task result payloads remain separate from lifecycle state, and panic
capture remains outside this implementation.


## Runtime metrics (point 052)

`RuntimeMetrics` provides opt-in atomic counters for submitted, started,
completed, failed, cancellation-requested, and cancellation-acknowledged events.
The API deliberately uses explicit `record_*` methods: the current executor and
scheduler do not update counters automatically, so callers must instrument the
corresponding event sites. A cancellation request and cancellation acknowledgement
are separate counters and must not be conflated.

Counters use relaxed atomic operations because they are statistics, not a
synchronization mechanism for task data. Increments saturate at `u64::MAX` to
avoid wraparound. `snapshot()` reads each counter safely, but the group is not a
globally atomic snapshot while concurrent updates occur. No task labels, timing,
external exporter, allocation, or dependency is introduced. Tests cover event
separation and concurrent increments.


## Structured tracing (point 053)

`TraceEvent` defines lifecycle event kinds for task submission, start, completion,
failure, and cancellation request/acknowledgement. `TraceBuffer` stores events in
recording order in a bounded, mutex-protected in-memory buffer. The caller chooses
a non-zero capacity. When full, recording rejects the new event and returns it
without overwriting existing history; callers may inspect or drain the buffer.

Tracing is opt-in and not automatically emitted by the executor or scheduler,
which currently lack task identity and a unified instrumentation hook. Do not
record while holding unrelated application locks. Snapshot/drain briefly lock the
buffer and copy or remove events; this implementation does not add timestamps,
span nesting, exporters, automatic task IDs, or external dependencies. Tests cover
ordered recording, overflow preservation, draining, and concurrent recording.


## Concurrency regression coverage (point 054, in progress)

The scheduler tests now include a barrier-based regression test that requires two
tasks to reach a synchronization point concurrently, then verifies the results
remain in FIFO dispatch order and the queue is drained. The barrier avoids relying
on wall-clock timing to infer overlap. Broader race-condition and scheduler stress
coverage remains tracked by points 055–057.
