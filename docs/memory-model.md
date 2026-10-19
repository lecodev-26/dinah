# dinah Memory — initial design model

Status: initial design; point 062 implements the first bounded buffer. The subsystem remains experimental and its API is not a stability guarantee.

## Goals

- Make the owner of each resource, its lifetime and the cost of reserving capacity explicit.
- Keep Rust's memory safety without `unsafe` in the initial implementation.
- Allow contiguous storage and reuse when a real workload demonstrates they are useful.
- Make limits, reservation failures and synchronization visible to the caller.
- Avoid performance promises without reproducible benchmarks.

## Ownership and lifecycle model

1. **Single ownership by default.** Values belong to an owning structure (`Vec<T>`, `Box<T>` or another suitable standard container). Destruction follows Rust's lifecycle and frees the resources owned by the container.
2. **Borrows for temporary access.** Views use references and slices with explicit lifetimes; they retain no references beyond what the compiler allows. A view acquires or transfers no storage ownership.
3. **Explicit transfer.** Moving a value transfers ownership without by itself duplicating the buffer. Copying elements or cloning storage must be explicit, documented operations.
4. **Exceptional, visible sharing.** When several consumers must share data, prefer borrows first; use `Arc<T>` for shared ownership and `Mutex<T>`/`RwLock<T>` only if synchronized mutability is needed. Synchronization is not added implicitly.
5. **No dangling references or manual freeing.** Do not expose raw pointers or allow a pool to recycle storage while valid views exist. The initial implementation uses no `unsafe` and frees no memory manually.

## Reservations, capacity and errors

- Standard Rust containers are the initial strategy; do not create a custom allocator in anticipation.
- APIs accepting limits must distinguish maximum capacity from current length and document what happens when the limit is reached.
- Bounded structures must prefer a recoverable error or returning the un-inserted element over truncating, overwriting or silently growing.
- Do not promise that a requested capacity can be physically reserved. If an API needs to report reservation failures, it must use a fallible operation when available and preserve the concrete error.
- An initial reservation may reduce reallocations, but it is not a guarantee of no future reservations unless the contract and operations ensure it.
- Sizes derived from external input must be validated before reserving and must avoid capacity calculations that overflow.

## Allocation abstraction (point 064)

**Decision: do not introduce a custom `Allocator` trait or a global allocator yet.** The only implemented storage uses the standard library's `Vec<T>` and `try_reserve_exact`, which already allows reporting reservation failures without resorting to a deliberately infallible allocation path. A custom interface now would duplicate capabilities without a second backend that demonstrates what common contract DINAH needs.

Initial contract:

- Memory types request and own storage through safe standard containers; they administer no addresses or manual freeing.
- Construction operations that need to reserve capacity use fallible paths where the library offers them. The public error describes the operation's failure, without exposing internal system allocator details.
- The requested logical capacity belongs to the container's contract; it is not interpreted as an exact byte budget or a resident-memory guarantee.
- Future abstractions should separate, if evidence requires it, the reservation mechanism from the container policy (limit, growth and recovery). A trait that claims to support incompatible allocators must not be added without a concrete use case and tests.
- Reconsider a trait when at least one real second backend exists (for example, an explicit pool) that shares operations and error semantics with standard storage. Then evaluate failures, alignment, ownership, concurrency, `no_std` compatibility if applicable and abstraction cost.

`BufferCreateError::AllocationFailed` is the current public boundary: it avoids leaking the allocator's concrete error and keeps the buffer's contract stable. This decision does not prevent introducing a richer abstraction at a later point; it avoids prematurely committing the API.

## Buffers, pools and reuse

- Own buffers own their elements; views are borrowed and do not own the memory.
- Pools and arenas are opt-in optimizations, bounded by an explicit capacity and recovery policy. They are not introduced as a global allocator or hidden behavior.
- A pool API must define who acquires and returns each resource, when it may be reused, what happens on exhaustion and whether the pool is thread-local or shared.
- Reuse must not change initialization semantics or expose stale values. Elements must be correctly initialized before being read.
- Discarding a pool frees its own resources; borrowed handles cannot outlive the storage owner.

## Pool contract (point 065)

**Decision: first design an explicit, bounded pool local to its owner; do not implement it yet or turn it into a global allocator.** Point 065 fixes the contract. Point 066 will separately resolve how to reset and clean each resource class before reuse.

Design contract for a first future implementation:

1. **Fixed, explicit capacity.** Creation takes a resource limit and fails recoverably if it cannot reserve/prepare what is required. The pool does not grow automatically on exhaustion and promises no exact byte budget unless a later API measures it explicitly.
2. **Visible exhaustion.** An acquisition with no resources available returns an exhaustion error/result; it does not block indefinitely, does not secretly allocate a resource and does not overwrite a lent resource.
3. **Exclusive lease with verifiable return.** The first API will prefer an RAII lease/handle representing the exclusive acquisition that returns the resource to the pool at the end of its life. The handle type must prevent the pool from being destroyed or the same slot from being acquired twice while the lease is alive. No indices or references will be returned that allow reuse without controlling the lifecycle.
4. **Clear ownership and failures.** The pool owns its slots and resources; the user temporarily owns the lease, not the underlying slot. If pool construction fails, no partial instance is published. If an acquisition cannot be satisfied, the pool's state remains valid and the error is observable.
5. **No implicit growth or recovery.** Capacity and the exhaustion policy are part of the contract. Releasing a lease allows re-acquiring the resource, but the reset/cleaning policy is defined in point 066 and it is not assumed that every type can be reused the same way.
6. **Explicit synchronization.** The first implementation will be local and unsynchronized: it promises no additional `Send`/`Sync` or automatic concurrent access. Shared use and its synchronization strategy are decided in points 071 and 073; there will be no implicit global mutex.
7. **Memory safety.** Keep the initial implementation in safe Rust, with no raw pointers or `unsafe`. An uninitialized slot must not be readable, nor recycled while a valid lease exists.
8. **No premature backend commitment.** The pool does not force introducing an `Allocator` trait. A common abstraction will only be extracted when another real backend demonstrates shared operations and semantics.

**Criteria for implementing later:** tests for empty and full pool, exhaustion, acquisition/return, destruction with available resources, lease exclusivity, construction failure and absence of access to uninitialized resources. Cleaning/reset details, concurrency tests and benchmarks belong to their specific points; they are not considered completed by this design decision.

## Memory reuse contract (point 066)

**Decision: reusing storage does not mean keeping previous values alive.** The default strategy will be to destroy values that no longer belong to the user and keep only the storage reservation where the container allows it. No generic `Reset` trait will be added, nor will it be assumed that any type can safely return to an initial state.

Rules for present and future APIs:

1. **Separate storage and value.** Reserved capacity may be reused even though previous values have been destroyed. Each value's lifetime follows Rust's rules; reserving memory does not equal initializing objects in it.
2. **`Buffer::clear` destroys, it does not reset.** It destroys exactly the currently contained elements and keeps the buffer's reservation. An element `Drop` may run observable effects. A later insertion must provide a new, correctly initialized value.
3. **`pop` transfers ownership.** The extracted element passes to the caller and no longer belongs to the buffer; the buffer may reuse the freed capacity for future insertions. The caller is responsible for the extracted value's lifetime.
4. **Do not expose stale content.** Views only cover the currently initialized elements. After clearing or extracting elements, no API may read them through the buffer or assume the reserved memory contains valid values. Uninitialized memory will not be exposed, nor will old bytes be used as new values.
5. **Explicit, specific reset.** If a future structure needs to keep an object alive and reset it, it must define an explicit operation whose contract describes which fields are restored, which resources are released, how errors are handled and which invariants remain guaranteed. It will not be applied automatically to arbitrary types.
6. **Pool slot reuse.** A returned lease marks the slot as available, but does not by itself authorize reusing the previous state. Before handing it to the next user, the pool must destroy and reconstruct the value or run a specific, documented and tested reset. The concrete policy is chosen per type and must not allow two simultaneous leases for the same slot.
7. **Failures without corrupt state.** If re-initializing a resource can fail, the API must represent that failure and keep the slot out of circulation until it returns to a valid state or is safely retired. A partially re-initialized resource must never be marked available.
8. **No zeroing guarantees.** Destroying a value does not guarantee physically erasing its bytes. Sensitive data requires a specific secure-erasure policy; `clear` or `Drop` will not be claimed to constitute cryptographic erasure.
9. **Measure before sophisticating.** Reuse must preserve observable semantics and will only be optimized when measurements indicate that avoiding constructions/allocations justifies the added complexity.

**Tests required when implementing:** count destructions with instrumented types; check that `clear` keeps capacity and allows inserting new values; verify that `pop` transfers exactly one value; test reset errors if introduced; and demonstrate that content of destroyed or uninitialized elements cannot be observed. The concrete pool, synchronization and benchmarks remain separate tasks.

## Alignment contract (point 067)

**Decision: alignment must come guaranteed by the type and the storage API used; DINAH will not promise special alignment without representing it explicitly in the contract.** The initial implementation stays in safe Rust and will not compute addresses or offsets manually.

Rules:

1. **Ordinary alignment.** For `Buffer<T>` and standard containers, trust the alignment guarantees of `T` and Rust's container. The consumer neither needs nor can assume additional alignment beyond what the type documents.
2. **No promises based on accidental layout.** Do not infer offsets, padding or stable layout from `repr(Rust)`. If a binary contract requires a concrete representation, it must choose and document an admitted representation, such as `repr(C)`, without confusing field layout with special alignment of a reservation.
3. **Greater-than-ordinary alignment.** If a real case demands explicit alignment (for example, a hardware interface), model it in a type or allocation API whose guarantee is verifiable. Construction must fail recoverably when the contract cannot be met; never return storage with insufficient alignment.
4. **Do not fake SIMD alignment.** A buffer being contiguous does not mean it starts at a specific SIMD boundary. Point 068 will separately evaluate concrete requirements of instructions, types and platforms; add no padding or speculative over-allocation without measurements.
5. **No manual pointer arithmetic.** Do not expose raw pointers or use `unsafe` to adjust addresses in this phase. If the available safe APIs cannot satisfy a demonstrated requirement, document the limitation and propose a separate design before expanding scope.
6. **Edge cases.** Any future implementation must define behavior for zero capacity and zero-sized types (ZST), plus large sizes/alignments and reservation failures. Do not deduce useful alignment from an address when no element storage exists.
7. **Alignment is not guaranteed performance.** Stricter alignment may increase cost or memory waste; it will only be adopted if representative benchmarks demonstrate an improvement without breaking the safety contract.

**Tests required when implementing special alignment:** check type guarantees with `align_of`/`align_of_val`; test ordinary and over-aligned types in the APIs that admit them; cover zero capacity and ZST; check that a construction failure publishes no partial instance; and verify the API claims no stronger guarantees than it actually offers. SIMD tests and comparative benchmarks remain in points 068 and 074–075.

## SIMD-compatible layout research (point 068)

**Research outcome: no universally optimal layout exists.** The choice must follow the access pattern and the data an operation actually processes. This point fixes design criteria; it adds no SIMD instructions, dependencies, vector types or new alignment guarantees.

### Layout alternatives

- **Array of Structures (AoS).** Each element contains all its fields (for example, `struct Particle { x, y, z, mass }`). It is suitable when most fields of each element are consumed. If an operation traverses only one field among many, the relevant data ends up interleaved and cache/vectorization efficiency may drop.
- **Structure of Arrays (SoA).** Each field occupies its own array (`x[]`, `y[]`, `z[]`, `mass[]`). It favors traversals that apply the same operation to many values of one field and can ease contiguous access and automatic vectorization. It may be less convenient when nearly all fields of a single element are needed together.
- **Array of Structures of Arrays (AoSoA).** It groups blocks of several elements and stores each field in arrays within each block. It can balance batch processing and per-entity grouping, but adds complexity, potential padding and a block-size choice dependent on workload and hardware. It will not be the initial option without evidence.
- **Contiguous single-type slices.** For simple operations over one field (`&[f32]`, for example), start with contiguous slices and ordinary safe loops. Let the compiler optimize before introducing intrinsics or a specific vector API.

### Decisions for DINAH

1. **Semantics and access pattern first.** Document whether an operation consumes whole entities, individual fields or blocks, and whether it inserts/removes elements frequently. Choose AoS, SoA or AoSoA based on that pattern.
2. **SoA is a candidate, not a global rule.** For numeric workloads traversing the same field of many entities, evaluate SoA first; keep AoS when operations use almost all fields of each entity or the design simplifies ownership and coherence.
3. **Contiguity does not imply vectorization.** A contiguous slice helps sequential access, but vectorization also depends on aliasing, dependencies between iterations, types, compilation options and architecture. Do not promise that a loop will be vectorized because of its shape or alignment.
4. **Do not require special SIMD alignment by default.** The base design respects the type's normal alignment. Only consider additional alignment/padding if a chosen API or instruction demands it and a measurement justifies it. Do not assume a fixed vector width or a universal block size.
5. **Keep a reference scalar implementation.** Correctness must not depend on SIMD being available. If a vector route is added later, compare its results against the scalar route, including sizes that are not multiples of the batch width and tail processing.
6. **Do not introduce a public SIMD API yet.** First identify a real operation and its data. Evaluate the target toolchain's stable capabilities when designing the implementation; any intrinsics, feature detection or CPU-specific route is outside this point.
7. **Measure reproducibly.** Compare layouts with the same data, algorithm, build configuration and execution conditions. Separate data conversion/reorganization time from kernel time; include small and large sizes. Record CPU, Rust version, flags, iteration count and variability. Memory benchmarks and optimization remain in points 074–075.

### Initial decision matrix

| Work pattern | Initial candidate | Risk to check |
|---|---|---|
| Operations consuming most fields of each entity | AoS | Unused fields and stride between equivalent values |
| Same computation over one field of many entities | SoA | Cost of reconstructing a whole entity and cross-array coherence |
| Batch processing needing entity and field grouping | Experimental AoSoA | Block size, padding and maintenance complexity |
| Numeric transformation over a homogeneous sequence | Contiguous slice and idiomatic scalar loop | Whether the compiler vectorizes and how it processes the remainder |

**Criterion for advancing to implementation:** choose a real operation, establish a correct scalar version, create representative data and measure at least two comparable layouts. Only add SIMD/alignment complexity when repeatable results demonstrate a useful improvement and a correct route for edge cases remains.

## Layout, alignment and concurrency

- Use the layout and alignment guaranteed by standard types and containers. Do not infer stable layout from `repr(Rust)` types or serialize their in-memory representation.
- Contiguous structures and data-oriented layouts will be chosen by measured access and workload, not aesthetic preference.
- SIMD-friendly layouts must respect hardware alignment and requirements without assuming every vector is aligned to a special boundary.
- Shared resources must satisfy the applicable `Send`/`Sync` contracts and document who synchronizes each mutation. A pool is not safe to share merely by being wrapped in a shared pointer.
- Avoid global locks as a default. Compare alternatives with benchmarks and concurrency tests before optimizing.

## Memory safety tests (point 072)

Memory tests must verify the observable contract without using `unsafe` or depending on internal allocator details:

- **Ownership and destruction:** values extracted via `pop` pass to the caller; `clear` destroys the remaining elements exactly once and allows reusing the reservation; destroying the container destroys all values it still owns. `SoaBuffer` applies these rules to both columns and keeps the logical row alignment.
- **Limits:** zero capacity, full capacity, rejected insertions and extensions that do not fit must not produce lengths above capacity nor lose accepted/rejected values.
- **Special types:** zero-sized types remain subject to logical capacity; over-aligned types keep the alignment Rust guarantees. No greater-than-type alignment is inferred.
- **Accesses:** views contain only initialized elements and Rust's borrows prevent incompatible mutations while a view is alive. The `compile_fail` example of `Buffer` validates this restriction in the documentation.
- **Implementation:** `#![forbid(unsafe_code)]` at the crate root prevents introducing `unsafe` blocks in project code. These tests reinforce the contract but do not replace Miri, fuzzing or an audit when more complex abstractions are incorporated.

## Explicit Memory synchronization (point 071)

The types of `dinah::memory` remain owners and include no implicit locks. When several threads need to mutate the same structure, it can be wrapped in `dinah::runtime::Shared<T>`, the existing primitive based on `Arc<Mutex<T>>`. Cloning `Shared<T>` shares the same value; it does not copy the buffer or its elements.

- `Shared::lock()` returns a mutex guard and communicates poisoning through `SynchronizationError::Poisoned`; the caller must handle that error explicitly. A potentially inconsistent state is not silently recovered.
- Keep guards as short as possible. Do not run external callbacks or wait for asynchronous work while holding a lock; define a consistent order if several locks are needed to avoid deadlocks.
- Operations of a composite structure —for example, inserting a row into both `SoaBuffer` columns— must run under the same guard to preserve invariants. Do not split the columns across independent locks.
- The wrapper does not make any `T` safe for any use: the `Send`/`Sync` restrictions still depend on Rust's types and the API used. The wrapper does not replace invariant validation or concurrency tests.
- Synchronization is opt-in. For immutable data, prefer sharing immutable references/values; for single-thread local access, use the buffer directly and avoid the mutex cost. It is not claimed that `Mutex` is the fastest strategy for every workload.

The concurrency tests also check that 16 threads starting through a barrier lose none of their 32,000 updates, and that a second thread cannot acquire the mutex while another holds the guard: after releasing the guard, acquisition completes. The test uses a timeout only to detect that the lock is still held and does not presume a thread scheduling order.

The `shared_soa_serializes_concurrent_row_insertions` test checks that several threads can insert complete rows through a `Shared<SoaBuffer<_, _>>` without misaligning the columns. These tests validate mutual exclusion and observable invariants, but do not demonstrate absence of deadlocks in any program, fairness, absence of starvation or scalable performance. Full benchmarks still belong to points 074–075.

## Initial buffer (point 062)

`dinah::memory::Buffer<T>` is an owning, contiguous container with fixed logical capacity. `try_with_capacity` fallibly reserves the maximum capacity before returning the buffer; therefore, a failed reservation leaves no partially initialized instance and later `try_push` calls need not grow the reservation while the limit is respected. The allocator's actual reservation may be larger than requested.

- Capacity is the maximum number of elements, not a byte limit; `T` may have different sizes.
- Zero capacity is valid. `is_full()` is true when length and capacity are both zero.
- `try_push` returns the original element in `BufferPushError::Full(T)` when full; it never overwrites or silently discards data.
- `as_slice` lends only the initialized elements; ownership remains in the buffer and the compiler prevents incompatible mutations while the view is alive.
- `Buffer<T>` does not implement `Clone`: it avoids implicit storage duplication whose reservation might not cover the logical limit. Ownership transfers through moves; duplicating elements requires an explicit consumer decision.
- `pop` transfers ownership of the last element to the caller and `clear` destroys the current elements while keeping the reservation. Destroying the buffer destroys all elements it still owns.
- The internal `Vec<T>` is not exposed: the consumer cannot bypass the logical limit through external mutation of the storage.
- There is no internal synchronization: mutable access requires `&mut self`.
- The API promises no byte memory limit and no guarantee that the physical reservation stays resident.

The implementation and its tests live in `src/memory/buffer.rs`. No mutable access to the internal storage or dynamic-growth API is exposed at this point.

## Initial contiguous structures (point 069)

The first implemented contiguous structure is `Buffer<T>`: it owns a contiguous sequence of initialized elements with a fixed maximum logical capacity. Point 069 extends its usefulness for batch workloads without duplicating storage or exposing the internal `Vec<T>`:

- `as_slice()` lends read-only access to the initialized elements; `as_mut_slice()` allows modifying those elements in place, but not changing the length, capacity or allocation.
- `try_extend_from_slice()` allows adding a sequence of `Clone` elements in a single logical operation. It checks the remaining capacity before modifying the buffer; if they do not all fit, it returns `BufferExtendError { requested, remaining }` and keeps the original content.
- Capacity remains a maximum of elements, not a byte budget. The constructor's reservation and the logical limit prevent growth beyond the limit in successful operations, but promise no exact physical size or memory residency.
- The API remains safe, with no raw pointers or `unsafe`, and adds no implicit synchronization. Borrows still govern when the buffer may be mutated.
- `try_extend_from_slice` needs `T: Clone`; cloning elements may run user code and panic. The no-modification guarantee applies to the normal capacity error, not to a transactional guarantee against `Clone` panics.

No second wrapper over `Vec<T>` or additional generic abstraction is created: first the existing structure is leveraged and reinforced. A contiguous matrix, SoA or another representation will be added when point 070 or a concrete workload defines dimensions, ownership and an access pattern that justify its API.

## Initial data-oriented structure (point 070)

`SoaBuffer<A, B>` implements a two-column (Structure of Arrays, SoA) structure on top of the existing bounded `Buffer<T>`s. Each column keeps its elements contiguous; a logical row is the pair of values at the same index. Both columns share capacity and length by construction, and the type exposes no mutable access to the internal buffers.

- `try_with_capacity` reserves both columns through fallible operations. If either fails, no partial structure is returned and Rust frees the already-created reservation.
- `try_push` adds both values as a logical row or returns both values without inserting them when capacity is reached. Equal capacities and private column control keep their lengths aligned.
- `as_slices` and `as_mut_slices` allow processing one field at a time without reconstructing rows or modifying the length. `pop` and `clear` operate on both columns to preserve index correspondence.
- `try_extend_from_slices` adds two slices as a batch: it validates that they have the same length and that the whole batch fits before mutating either column. It reuses the reservations and avoids running the capacity check per row. It requires `Clone` for both types and promises no rollback if a clone panics.
- Zero capacity is valid; the structure does not grow automatically, synchronizes internally or use `unsafe`. The API promises no special SIMD alignment or performance improvement without benchmarks.
- SoA is convenient when one field of many rows is processed repeatedly. For workloads consuming almost all fields of a row, AoS may be simpler or more efficient; AoSoA is deferred until a workload and measurements justify it.

The implementation and tests live in `src/memory/soa.rs`. It is a generic primitive limited to two columns, not an ECS API or a claim of universally optimal layout.

## Contracts the following points must resolve

| Point | Pending decision |
|---|---|
| 062 | Completed: first contiguous, bounded buffer; other buffer types remain subject to future needs |
| 063 | Completed: single ownership of `Buffer<T>`, borrows with compiler lifetimes, transfer by move, no implicit cloning or mutable access to the internal container |
| 064 | Completed: use standard containers and fallible reservations; defer a custom trait until a real second backend exists |
| 065 | Completed: bounded pool contract, explicit exhaustion, exclusive leases and local synchronization by default |
| 066 | Completed: separate storage reservation from value lifetime; `clear` destroys, `pop` transfers ownership and reset must be explicit |
| 067 | Completed: use type/container alignment guarantees; special alignment only with an explicit contract, recoverable errors and tests |
| 068 | Completed: AoS, SoA, AoSoA and contiguous slices documented; SIMD implementation and benchmarks remain separate |
| 069 | Completed: contiguous `Buffer<T>` with slice access and bounded batch extension; no implicit growth |
| 070 | Completed: `SoaBuffer<A, B>` with two contiguous columns, coordinated rows and explicit limits |
| 071–073 | Synchronization, invariants and safety/concurrency tests |
| 074 | Completed: reproducible benchmark `benches/memory.rs` for preallocated push, batch extension, SoA insertion/scan and `Shared` cost with a median of 7 samples; each run's results are saved as a CI artifact |
| 075 | Completed: `SoaBuffer::try_extend_from_slices` validates the batch and avoids per-row checks; comparison with precomputed entries in CI (7 samples): 2,031 vs 6,109 ns/row (64), 0.548 vs 2.495 (1,024) and 5.837 vs 8.118 (16,384). Raw: `artifacts/benchmarks/memory-soa-batch-2026-10-11.txt`; runner-specific results, no universal guarantee |
| 076 | Integrated documentation and subsystem exit criteria |

## Criteria for accepting an implementation

- Documented API with clear ownership, lifetimes, limits, errors and concurrency guarantees.
- Tests for limits, empty buffers, failures, reuse, destruction and shared accesses.
- No `unsafe` in the initial scope except a separate, motivated, reviewed and tested proposal.
- Remote CI green for formatting, tests, Clippy and documentation.
- Any performance-improvement claim backed by a repeatable benchmark and a before/after comparison.

## Out of scope for now

Custom allocators, arenas, pools, SIMD, FFI and real-time guarantees are not implemented here. The initial `Buffer<T>` is a minimal experimental primitive, not a complete Memory subsystem. The following points must justify and test each abstraction before incorporating it.

## Practical guide

For usage examples of `Buffer<T>` and `SoaBuffer<A, B>`, public errors, capacity, batches, ownership and links to archived results, see the [Memory practical guide](memory.md). This document keeps the design decisions and invariants; the practical guide describes the available API.
