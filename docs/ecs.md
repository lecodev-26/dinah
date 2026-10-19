# dinah ECS — initial design

Status: initial architecture implemented and documented; `Entity`, `Component`, the `World` lifecycle, per-`TypeId` typed storage, individual component access and explicit Runtime/Compute adapters are available. The combined-query API and safe parallel system execution remain pending. The decisions in this document are revisable and do not constitute a public stability guarantee.

## Goals

- Represent entities as lightweight identifiers, with no embedded domain data.
- Store typed components in contiguous storage, separated by type, with safe access and no `unsafe`.
- Make ownership, errors, deletion, ID reuse and iteration semantics explicit.
- Allow readable queries and later offer explicit integration with Runtime and Compute.
- Measure real workloads before introducing archetypes, code generation, SIMD or automatic parallelism.

Out of initial scope: reflection/automatic serialization, integrated hierarchical relationships, global events, a system scheduler, derive macros, dynamic plugins and real-time guarantees.

## Initial decision: typed sparse sets

The first version will use independent storage per component type, implemented with contiguous vectors and a sparse index to locate an entity. An archetype/table design will not be adopted initially: moving rows between tables when composition changes adds movement and borrowing complexity before having representative benchmarks. The decision may be revisited if measured workloads favor composition queries over per-type access.

A conceptual `ComponentStorage<T>` contains:

- `entities: Vec<Entity>`: entities in the same order as the components.
- `components: Vec<T>`: contiguous values, aligned by index with `entities`.
- `sparse: Vec<Option<usize>>`: row index for each known entity index; the entry is also validated against the generation.

When a row is deleted, `swap_remove` is used; the last row moves to the vacant position and its sparse index is updated. Therefore, iteration order is not guaranteed after structural mutations. A storage's slices are contiguous, but components of different types are not promised to be together.

## Entity identity and lifecycle

`Entity` will be a small value with `index` and `generation` (both unsigned integers). The generation prevents a user-held identifier from accidentally referring to another entity after its index is destroyed and reused.

- Creation is administered by `World`; the user is not asked to invent indices and `CoreId` is not reused as an allocator.
- `World` keeps generations and a free list of reusable indices.
- Validity requires the index to exist, the generation to match and the entity to be alive.
- Destroying an entity invalidates the handle, removes its components from all storages and only then allows the index to be reused with a new generation.
- Generation overflow must have an explicit, tested policy; it must never allow an old handle to silently validate. The implementation may retire that index if it cannot increment the generation.
- Handles are local to a `World`; they imply no global uniqueness or stability across processes, serializations or sessions.


### Implemented contract: `World` (point 094)

`dinah::ecs::World` is the single owner of the entity table and its typed storages. `spawn()` returns `Result<Entity, WorldError>`; `despawn(entity)` invalidates a live handle, first removes its components from all storages and returns `EntityNotAlive` for nonexistent, stale or already-destroyed handles. `contains`, `len` and `is_empty` allow inspecting the state. Freed indices are reused with an incremented generation; if a generation reaches `u32::MAX`, the slot is retired to prevent wraparound. Index-space saturation returns `EntityIndexExhausted`. Unit and integration tests cover liveness, counts, reuse, stale-handle rejection and overflow retirement.

### Explicit system adapter to Runtime (point 098)

`World::run_system(&mut self, &Scheduler, closure)` allows running a system operation that receives exclusive access to the `World` through the public Runtime. In Runtime v0.1, `Scheduler::schedule` executes the closure immediately on the calling thread; therefore the adapter creates no threads, holds no world and promises no background execution. The nested result distinguishes a scheduler rejection (`ScheduleError`) from the system's own error. The exclusive borrow prevents two mutable closures from running simultaneously over the same `World` through this API. The integration is explicit and synchronous; parallel systems are outside this point.

### Typed access metadata (point 100, in progress)

`SystemAccess` allows declaring `read::<T>()` and `write::<T>()` per component type and querying the declared mode. Repeated declarations are deduplicated; if a type appears as both read and write, `Write` prevails, which is the conservative interpretation. `conflicts_with` returns a conflict if both systems declare the same type and at least one writes it; read/read and distinct types are compatible. The descriptor is public metadata: it does not verify that a closure respects its declaration, does not lend components and does not execute systems. Therefore it must not yet be used as sufficient proof to launch workers. Unit and public-contract tests cover read/read, read/write, write/write, distinct types, repeated declarations and `!Send`/`!Sync` components.

### Deterministic metadata planner (point 100, in progress)

`SystemPlan::from_accesses` creates phases of system indices deterministically and preserves input order. It uses a conservative contiguous-batching strategy: it adds a system to the current phase if it does not conflict with any of its members; if there is a conflict, it opens a new phase. Thus, conflicting systems keep their relative order and compatible accesses can share a phase. The API exposes `batches()`, `len()` and `is_empty()`, and accepts empty input without creating phases. The tests verify grouping, order and repeatability. This planner operates only over declarations: it does not run closures, does not prove they match their declarations, does not detect structural operations and does not make `World` `Send`/`Sync`; therefore it does not yet enable safe parallel execution.

### Safety contract for parallel systems (point 100, in progress)

Parallel execution of ECS systems is not enabled yet. Runtime's scheduler can run independent closures in parallel, but that does not make a `World` a safe object to share: `World` is single-owner, `Component` admits types that are not `Send`/`Sync`, and component storage is not yet connected to `World`. Passing the same `&mut World` to several workers or capturing shared components without borrow analysis would be incorrect.

Before implementing execution, each system must explicitly declare the component types it reads and writes. Two systems may only belong to the same parallel batch if their access sets are compatible: read/read of the same type is compatible; write/read, read/write and write/write of the same type are conflicts. A missing, dynamic or ambiguous access declaration must block parallel execution, not assume safety. Components that cross worker boundaries will have to satisfy the `Send`/`Sync` bounds required by the concurrent API, without thereby restricting the general `Component` contract for sequential use.

The future implementation must additionally guarantee that structural operations (`spawn`, `despawn`, component insertion/removal) do not invalidate active references. Initially they may run in a separate exclusive phase from the component read/write phases. Parallel batches must be deterministic in selection and result presentation, even if the temporal completion order is not stable. The worker count will be explicit and no speedup will be promised for small workloads.

**Acceptance criteria:** typed descriptor, basic conflict detector and deterministic contiguous-batch planner implemented; missing is an execution API that validates declarations and does not allow sharing `World` mutably across workers; boundary tests (repeated types, read/write access, empty batch and invalid worker count); and cross-platform CI before marking point 100 complete. Until then, `World::run_system` remains synchronous and there is no public API for parallel ECS systems.

### Slot, generation and storage consistency (point 101)

The consistency tests check internal invariants after sequences of mutations, not just the result of an isolated operation. For `World`, the `alive_count` counter must match the number of live slots; every index in the free list appears exactly once and references a slot that is neither live nor retired; every slot that is not live and not retired must be in the free list; and live or retired slots cannot be free. The generation is never validated when the slot is not alive.

For `ComponentStorage<T>`, the `entities` and `components` columns must keep the same length. Every stored entity must point from its sparse index to its current row, and every present sparse entry must reference a valid row whose entity index matches the sparse position. The tests verify these properties after insertions, replacements, `swap_remove` deletions and reuse of the same index with a different generation. `World` registers storages by `TypeId` and removes the entity's row from every storage during `despawn`; the public tests verify that no components survive destroying and reusing an entity.

### Implemented contract: `Component` (point 093)

`dinah::ecs::Component` is a public marker trait with `Component for T` for every `T: 'static`. It provides no methods and requires no derive/manual boilerplate; it makes the conceptual bound explicit in future storage and `World` APIs. The blanket implementation is intentional: the initial contract is the lifetime required by `Any`/`TypeId`, not the ability to send or share the value across threads. Unit and integration tests check common types and `Rc<Cell<u32>>`, which is neither `Send` nor `Sync`.

### First implemented contract: `Entity` (point 092)

`dinah::ecs::Entity` is a `Copy` handle with two private `u32` fields: index and generation. It exposes `index()` and `generation()` as read-only accessors and a diagnostic `indexvgen` representation through `Display`. Its constructor is private to external users; the `World` will be responsible for issuing valid handles. The implementation assigns no IDs, does not by itself prove that a handle is alive and promises no uniqueness outside the owning world.

The tests verify that equality and hashing distinguish different generations of the same index, that equivalent handles compare equal, that maximum values are preserved and that the handle occupies eight bytes on Rust's target platform (two `u32`). Liveness validation and reuse are performed by `World`; the handle alone does not demonstrate that the entity is still alive.

### Contiguous storage and iteration (points 096–097)

`ComponentStorage<T>` is an internal ECS detail with three vectors: `entities` and `components` form contiguous row-aligned columns; `sparse` maps the entity index to a candidate row. Before returning or removing a value, the full `Entity` identity, including the generation, is checked. Replacing the same entity's component keeps the row; reusing an index with a different generation removes the old row and adds the new one. When a row is deleted, `swap_remove` is used and the moved row's sparse index is updated.

Insertions reserve capacity in every column that might grow before modifying their contents. A reservation failure returns `StorageError::AllocationFailed` and does not leave the columns with different lengths. The storage cannot by itself validate a handle's liveness; `World` validates index and generation before delegating access. The initial integration exposes `World::insert`, `get`, `get_mut`, `remove` and `has`, and `despawn` removes the associated components. Combined queries remain pending.

Point 097 adds `iter()` and `iter_mut()`, which linearly walk the two aligned columns and produce `(Entity, &T)` or `(Entity, &mut T)` pairs without looking up each row in the sparse index. Mutable iteration only grants mutable borrows of distinct rows and offers no structural operations over the storage. Both traversals follow the current physical row order, which may change after insertions or deletions; it is not a stable entity order. The tests check aligned pairs, empty storage and mutable updates without altering the columns' shape.

## Components

A component is any `'static` Rust type. Point 093 exposes `dinah::ecs::Component` as a marker trait with a blanket implementation for all `'static` types: users need not implement anything manually and the trait serves as a readable bound in ECS APIs. `Send + Sync` is not required: `World` is initially single-owner and types like `Rc<Cell<T>>` remain valid components. If a future concurrent API needs stricter bounds, it will impose them on that API instead of prematurely restricting all components.

Each `TypeId` identifies the type's storage in a `World`. Storage will be kept behind a private type-erased registry (`Any`) and every typed access will check the downcast; a type disagreement is treated as an internal error/invariant, never through raw pointers. No references that outlive an incompatible structural mutation of the `World` will be exposed.

## World and available operations

`World` is the owner of entities, generations, the free list and storages. The implemented public API includes:

- `spawn` / `despawn` to create and destroy entities.
- `insert<T>` to add or replace a component.
- `get<T>` / `get_mut<T>` to access an existing component.
- `remove<T>` to remove a component.
- `has<T>` to check a component's presence; `contains(entity)`, `len()` and `is_empty()` for entity liveness.

`insert` returns `Result<Option<T>, WorldError>`: an `Ok(Some(old))` indicates it replaced a previous value. The `get`, `get_mut` and `remove` accesses return `None` if the handle is dead/stale or lacks that component. `WorldError` distinguishes `EntityNotAlive`, `EntityIndexExhausted` and `AllocationFailed`. `World` remains single-owner and not internally synchronized.

## Query contract (point 095; API still pending)

This section fixes the intended semantics for a future query API; it does not claim that combined queries are implemented. Individual component access and storage are implemented, but a public API to traverse entities satisfying several component types does not exist yet.

### Selection and results

- A query selects live entities that have all required component types. An entity lacking any of them does not appear in the result.
- The first version is limited to a small, explicit combination of typed components; it includes no arbitrary filters, dynamic expressions, general joins or code generation.
- The queried set belongs to a single `World`. Returned `Entity` values are handles of that world and must be validated by index, generation and liveness.
- Queries create no entities or components and perform no implicit structural changes.
- A query with no matches is a valid empty result, not an error.

### Borrows and mutability

- Read-only queries may expose `&T` while the shared borrow of `World` is held.
- A query with writes must express the mutable components in its API type and exclude mutable aliasing. Combining incompatible accesses to the same type via duplicated query parameters will not be allowed.
- While references from a query exist, the type system must prevent structural mutations of the `World` that would invalidate those references.
- If the generic API to combine borrows cannot be implemented safely in stable Rust, we will start with a smaller surface (read-only included) rather than introducing `unsafe` or promising an undemonstrated capability.
- No `Send`/`Sync` is promised for query results: the initial components and `World` are single-owner, and parallelism will be defined separately.

### Order and complexity

- Result order is not part of the stable contract and may change after insertions, deletions or row reuse.
- The initial implementation may walk one driving storage and check the other components; choosing the smallest storage is a later optimization, not an observable guarantee.
- Queries promise no constant complexity per entity or specific performance until the storage, iteration and benchmark points provide evidence.

### Cases the tests must cover

1. Full match, missing component and empty query.
2. Destroyed entities and stale handles excluded after reusing an index.
3. Several matching entities and components aligned with their entity, even after `swap_remove`.
4. Coexisting read borrows and compile-time rejection of incompatible accesses when the API shape allows it.
5. Absence of structural changes as a side effect of iterating.

These cases are acceptance criteria for the future implementation; point 095 is closed as a design contract, not as a query implementation.

## Safety limits and errors

The crate keeps `#![forbid(unsafe_code)]`. The implementation will prefer `Vec`, `HashMap<TypeId, Box<dyn Any>>` and safe downcasts, with the `borrow checker`'s restrictions visible in the API. Operations that may fail due to invalid input must return typed errors. A memory allocation failure must not leave a `World` with out-of-sync entities or columns; operations needing several mutations must validate and reserve before committing changes when possible.

Destroying an entity must leave all storages coherent. Tests will cover components across several storages, rows moved by `swap_remove`, index reuse and stale-handle rejection. Destructor panics follow Rust semantics and no transactional recovery against a panic is promised.

## Future integration

- **Memory (096):** reuse the lessons of `Buffer<T>` and `SoaBuffer<A, B>`, but do not force the current fixed buffer to be a dynamic-size storage without evidence that its contracts fit.
- **Compute (099):** `ComponentStorage::for_each_mut` reuses `compute::for_each` over the contiguous component column and pairs each element with its `Entity` at the same index. The operation is sequential, does not allow structural mutations from the callback and preserves the columns' size/alignment. The current integration of storages with `World` allows typed individual access; public iteration/queries and parallel partitions remain pending. No parallelism or speedup is promised.
- **Runtime (098, 100):** `World::run_system` explicitly connects a synchronous system with `Scheduler`. Point 100 remains in progress: the safety contract requires explicit declarations of read/written components, rejection of conflicting batches and isolation of structural changes; there is still no implemented parallel ECS execution. No hidden scheduling is introduced into `World` operations.
- **Consistency (101):** invariants for generations, the free list, column alignment and sparse indices, and absence of components on dead entities.
- **Benchmarks (102–103):** compare sparse sets against a simple baseline, record entity/component size, composition, reads/writes, insertions/deletions and query distribution. Do not claim universal superiority or adopt archetypes purely on intuition.

## Acceptance sequence

1. **091 — ECS design:** decisions, invariants, limits and integration sequence documented here.
2. **092–094 — Entity, Component, World:** handles, the public bound and `World`'s initial lifecycle implemented.
3. **095–097 — Queries, storage and iteration:** the query contract, contiguous storage and safe internal iterators implemented/documented; the integration of individual storage access with `World` was added later. Public queries and public world iteration remain pending.
4. **098–101 — Runtime/Compute integration, parallel systems and consistency:** the explicit synchronous adapter from `World` to Runtime and the storage's internal sequential integration with `compute::for_each` are implemented; point 100 has the safety contract, `SystemAccess`, basic conflict detection and the deterministic `SystemPlan` implemented; it remains in progress until safe execution is available that validates declarations, isolates structural changes and passes cross-platform CI. Point 101 adds invariant tests for slots/free list and the sparse index/columns under repeated mutations.
5. **102–104 — Benchmarks, memory access and documentation:** measure before changing representation or promising performance.

### Baseline benchmark (point 102, completed)

`benches/ecs.rs` measures the public API available today: entity creation, destruction, slot reuse and `contains` traversal, with sizes of 64, 1,024 and 16,384 entities and seven samples per case. The methodology and limits are documented in `docs/ecs-benchmarks.md`. The first CI run completed successfully on all five platforms and its results are summarized in `docs/ecs-benchmarks.md`. It measures no components or queries because this benchmark corresponds to point 102 and ran before integrating storage with `World`; it remains solely the entity-lifecycle baseline.


### Initial component integration with World

`World` keeps a private `TypeId -> Box<dyn ErasedStorage>` registry; each concrete storage still uses `ComponentStorage<T>` and its contiguous columns. The current public API includes `insert<T>`, `get<T>`, `get_mut<T>`, `remove<T>` and `has<T>`. `insert` only accepts live entities, returns the replaced component and propagates `WorldError::AllocationFailed` if column reservations fail. `get`/`get_mut`/`remove` do not access dead or stale handles. `despawn` walks the registered storages and removes that entity's component before invalidating its slot; reusing the index leaves no components from the previous generation.

This integration does not yet implement multi-component queries, public iteration over `World`, parallel system execution or `Send`/`Sync` guarantees. The storage map and component types are not exposed directly, and the API remains single-owner and safe without `unsafe`.


### Example using the current public API

```rust
use dinah::ecs::World;

let mut world = World::new();
let entity = world.spawn()?;
world.insert(entity, 42_u32)?;
assert_eq!(world.get::<u32>(entity), Some(&42));
*world.get_mut::<u32>(entity).expect("component exists") += 1;
assert_eq!(world.remove::<u32>(entity), Some(43));
world.despawn(entity)?;
# Ok::<(), dinah::ecs::WorldError>(())
```

The example illustrates only already-available operations; it does not depend on a not-yet-existing query API. In application examples, creation/insertion errors must be propagated or handled according to the program's policy.
