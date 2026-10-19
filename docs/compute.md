# dinah Compute — sequential API and evolutionary design

Status: the sequential primitives, batches, parallel iteration and reduction, and the first pipeline API are implemented and documented. Later optimizations and stages remain future work; they must not be interpreted as available APIs.

## Goal and limits

Compute will offer data-processing operations that can run first sequentially and, when a measured need exists, in parallel. It must reuse Core's contracts and Runtime's and Memory's primitives without merging their responsibilities.

- **Core** validates shared types and contracts; it runs no kernels.
- **Memory** owns contiguous storage and fixes its capacity limits; it does not decide how data is processed.
- **Runtime** executes and coordinates work; Compute defines the semantics of data operations.
- **Compute** offers pure or explicitly mutable operations over borrowed data, with clear error, ordering and concurrency limits.

The first version will be small, generic and free of external dependencies. There will be no DSL, task graph, JIT compiler, custom allocator or real-time guarantees.

## Staged API design

### Stage A — sequential primitives

The initial API uses generic functions over borrowed slices, with no global state. `for_each` and `try_reduce` allocate no memory of their own; `map` explicitly allocates the output `Vec`.

```rust
pub fn for_each<T, F>(items: &mut [T], operation: F)
where
    F: FnMut(&mut T);

pub fn map<T, U, F>(items: &[T], operation: F) -> Vec<U>
where
    F: FnMut(&T) -> U;

pub fn try_reduce<T, A, E, F>(
    items: &[T],
    initial: A,
    operation: F,
) -> Result<A, E>
where
    F: FnMut(A, &T) -> Result<A, E>;
```

These functions are part of the public `dinah::compute` API. All three operations are sequential and preserve input order. `try_reduce` propagates the user's first error and stops; `for_each` offers no rollback if the closure panics. A future `map` variant with a caller-provided buffer will be evaluated only if needed.

### Stage B — explicit batches

The sequential batch API is implemented as `for_each_batch`. It takes a positive size and hands the closure borrowed mutable slices of the original input. It creates no temporary buffers or per-element tasks. The last batch may be smaller than the requested size.

```rust
pub enum BatchError {
    ZeroSize,
}

pub fn for_each_batch<T, F>(
    items: &mut [T],
    batch_size: usize,
    operation: F,
) -> Result<(), BatchError>
where
    F: FnMut(&mut [T]);
```

A zero size returns `BatchError::ZeroSize` before invoking the closure or mutating elements. With an empty input and a valid size, the closure is not invoked. Batches and elements preserve the original order. No rollback is promised if the closure panics; the panic propagates.

### Stage C — parallel iteration and reduction

Parallel iteration is implemented through `for_each_parallel` and explicitly uses `Runtime::Scheduler::run_parallel`; it keeps no hidden pool and creates no independent execution system. Partitions are disjoint mutable slices and are delivered as tasks to Runtime's queue. Workers are bounded by `worker_count` and by the number of elements; they are scoped and joined before the call returns.

```rust
pub enum ParallelError {
    ZeroWorkers,
}

pub fn for_each_parallel<T, F>(
    items: &mut [T],
    worker_count: usize,
    operation: F,
) -> Result<(), ParallelError>
where
    T: Send,
    F: Fn(&mut T) + Sync;
```

A `worker_count` of zero returns `ParallelError::ZeroWorkers` before mutating the input. An empty input with a positive value creates no workers. Each element is processed exactly once in a normal run, but completion order is undefined. `T: Send` allows moving partitions to workers and `F: Sync` allows sharing the operation safely. Panics propagate according to Runtime's contract after the current batch's tasks are joined; prior mutations are not rolled back. No speedup is guaranteed for small workloads.

Parallel reduction is implemented as `try_reduce_parallel`. Each partition starts with a copy of `identity`; its partial accumulators are combined sequentially in partition input order. For partition-independent results, `identity`, `operation` and `combine` must respect the stated algebraic contract (including associativity); floating-point operations may vary due to regrouping.

```rust
pub enum ParallelReduceError<E> {
    ZeroWorkers,
    Operation(E),
}

pub fn try_reduce_parallel<T, A, E, F, C>(
    items: &[T],
    worker_count: usize,
    identity: A,
    operation: F,
    combine: C,
) -> Result<A, ParallelReduceError<E>>
where
    T: Sync,
    A: Clone + Send,
    E: Send,
    F: Fn(A, &T) -> Result<A, E> + Sync,
    C: Fn(A, A) -> A + Sync;
```

Zero workers is rejected before running the closures; empty input returns `identity`. All partitions complete before the result is returned. If several partitions fail, the error of the first partition in input order is returned, not the temporally first error observed. Combination only happens if all partitions succeeded. Panics follow Runtime's contract.

- Parallel iteration promises no completion order; it does guarantee that each element is processed exactly once in a normal run.
- Parallel reduction requires an explicit, associative combine operator to obtain partition-independent results. Bit-for-bit equivalence for reordered floating-point operations will not be promised.
- Task errors propagate without being silently converted into default values. The policy for aggregating multiple errors will be defined before exposing the API.
- Panics follow Runtime's documented policy; Compute must not swallow them.
- Cancellation will be cooperative and cannot safely interrupt an arbitrary closure.

### Stage D — pipelines

The first version is implemented as `ComputePipeline<'a, T>`, a chain of mutable stages over a single borrowed slice. `ComputePipeline::new(&mut items)` takes an exclusive mutable borrow; `for_each`, `for_each_batch` and `for_each_parallel` run each stage immediately and return `Result<&mut Self, PipelineError>` so they can be chained. `finish(self)` consumes the pipeline and returns the original slice borrow.

```rust
use dinah::compute::ComputePipeline;

let mut values = [1, 2, 3, 4];
ComputePipeline::new(&mut values)
    .for_each(|value| *value += 1)?
    .for_each_batch(2, |batch| batch.reverse())?
    .for_each_parallel(2, |value| *value *= 2)?;
# Ok::<(), dinah::compute::PipelineError>(())
```

- **Execution:** immediate, in call order; no closures are stored and work is not deferred.
- **Ownership:** the pipeline holds an exclusive mutable borrow of the input. It owns and creates no intermediate collection and allocates no memory of its own.
- **Errors:** `PipelineError::Batch` and `PipelineError::Parallel` preserve the corresponding stage's configuration error. A failed stage cuts the chain: the following ones do not run. Prior mutations are not rolled back.
- **Parallelism:** it only occurs in the stage that explicitly invokes `for_each_parallel`; it uses Runtime and preserves that function's concurrency contract.
- **Limits:** this first version transforms the same slice in place. `map` and reductions remain independent functions because they produce output values/accumulators with distinct ownership. There is no scheduling, deferred cancellation or automatic reordering.
- **Panics:** they propagate according to the executed primitive's contract; no rollback is promised.

## Cross-cutting contracts

1. **Ownership:** prefer borrowed slices; output is only allocated when the signature explicitly says so.
2. **Mutability:** read operations receive shared references; write operations receive exclusive mutable access or disjoint partitions.
3. **Errors:** validate invalid parameters before starting when feasible; preserve user errors.
4. **Determinism:** document visit order, result order and reduction behavior separately.
5. **Panics:** do not hide panics or promise rollback of mutations that already happened.
6. **Concurrency:** no operation is automatically parallelized in the first stage; explicit parallelism avoids cost surprises.
7. **Performance:** measure overhead, allocations, small and large sizes; introduce no SIMD or scheduler complexity without evidence.
8. **Safety:** keep `#![forbid(unsafe_code)]` until a concrete, audited and measured proposal justifies revising that policy.

## Implementation and acceptance sequence

- **077 — Design Compute API:** responsibilities, operation families and contracts documented; no nonexistent functions are declared.
- **078 — Define processing primitives:** implemented with `for_each`, `map` and `try_reduce`, unit tests and public documentation examples.
- **079 — Create batch processing:** implemented with `for_each_batch` and `BatchError`; zero size validated before mutating, partial final batch and borrowed slices with no temporary buffer.
- **080 — Create parallel iteration:** implemented with `for_each_parallel`, disjoint partitions, `Scheduler::run_parallel`, coverage tests, empty input and zero-worker rejection.
- **081 — Create parallel reduction:** implemented with `try_reduce_parallel` and `ParallelReduceError`; combination in partition order, first error by input order and documentation of identity/associativity/determinism.
- **082 — Create pipelines:** implemented as an immediate `ComputePipeline` over a borrowed slice, with sequential, batch and parallel stages; typed errors, stop-after-failure, no hidden buffers or deferred execution.
- **083–089:** research SIMD, optimize layouts, instrument and compare sequential/parallel, and test determinism with archived results.
- **090 — Document Compute:** a user manual based on already implemented APIs, not proposals.

Remote CI validates formatting, tests, Clippy and public documentation. Heavy benchmarks and performance comparisons must be archived under `artifacts/benchmarks/` with command, environment and date; numbers are never invented to close a point.

## Tested determinism (point 089)

`tests/compute_determinism.rs` validates the public contract across repeated runs: an associative integer sum returns the same result with 1, 2, 3, 7 and 64 workers; parallel iteration produces the same indexed vector even though task completion order is undefined; and a reduction that fails in several partitions always returns the error of the earliest partition in input order. It also repeats the sequential visit order, `map`'s output order, batch splitting with a partial final batch, zero-size rejection without mutations or callbacks, `try_reduce` stopping at the first error, and `ComputePipeline` halting when a stage fails. Empty-input cases are checked explicitly. These tests imply no temporal determinism, completion ordering, or bit-for-bit equivalence for floating-point reductions.

## Sequential/parallel and SIMD benchmarks (points 083–088)

`benches/compute.rs` compares the direct scalar loop, `compute::for_each`, `compute::for_each_parallel` with 2 and 4 workers, and the safe `wide::f32x4` candidate over the same numeric kernel and identical sizes. Buffer setup happens outside the samples; each variant repeatedly transforms its own buffer and the benchmark verifies scalar/parallel result equality before reporting times. 11 samples per case are recorded and the median reported. The cross-platform release run (run 38208860456, code `f728dba482d00450bb06a7e7b5f994e3f82152dc`) is archived in [`artifacts/benchmarks/f728dba482d00450bb06a7e7b5f994e3f82152dc/run-38208860456/`](../artifacts/benchmarks/f728dba482d00450bb06a7e7b5f994e3f82152dc/run-38208860456/).

### Measured result: lightweight sequential/parallel kernel (point 088)

At the maximum size of 262,144 elements, the medians of `for_each_parallel` with 2 workers were between 4.0× and 10.3× the time of `for_each` across the five runners; with 4 workers, between 6.3× and 16.6×. At 16,384 elements, the parallel routes were also slower on all runners. For sizes of 64 and 1,024, coordination cost dominates by far. No break-even point appears for this concrete workload: the sequential route is the right choice and the parallel one must not be activated automatically for lightweight operations.

| Runner | Sequential, 262,144 elems. | 2 workers / sequential | 4 workers / sequential |
|---|---:|---:|---:|
| Linux x86-64 | 18,131 ns | 6.7× | 10.1× |
| Linux ARM64 | 30,056 ns | 4.1× | 6.5× |
| macOS ARM64 | 18,545 ns | 4.0× | 6.3× |
| macOS x86-64 | 37,095 ns | 10.3× | 16.6× |
| Windows x86-64 | 20,780 ns | 6.2× | 11.6× |

The figures correspond to a microbenchmark of lightweight arithmetic transformation, not a representative application or a universal estimate of parallel performance. The result closes the comparison task but does not invalidate parallelism for kernels with much more work per element, batch amortization or a different coordination strategy. The complete artifacts include the eight sizes and the environment data.

The research on SIMD alternatives and its restrictions is recorded in [`docs/simd-research.md`](simd-research.md). No manual SIMD is added and no speedup is claimed: point 084 remains in progress until there is a production kernel that justifies the complexity and reproducible evidence. No automatic vectorization or special alignment is promised.

## Deferred decisions

Open until a use case and tests justify them: kernel traits, configuration types, optimal batch size, per-operation cancellation policy, representation of parallel errors, floating-point reduction strategy, deferred pipelines, SIMD and external dependencies. These abstractions must not be anticipated just to expand the API.
