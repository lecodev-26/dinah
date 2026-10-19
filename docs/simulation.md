# dinah Simulation — initial API design

Status: initial design completed (105), fixed clock (111), deterministic update (112), initial simulated-entity collection (113), identity integration with ECS (114), traversal via Compute (115) and the deterministic replay test (116) implemented. `dinah::simulation` offers `SimulationClock`, `SimulationTime`, `SimulationTimeError`, `Simulation<S>`, `SimulationStepError`, `SimulationEntityId`, `SimulationEntities<T>`, `SimulationEntitiesError`, `SimulationEntityBindings` and `SimulationEntityBindingError`. `SimulationEntities::for_each_mut` uses Compute's sequential traversal to update states in insertion order; parallel execution from Simulation remains explicit and is not activated automatically.

## Goal and responsibilities

Simulation will model the explicit advancement of a simulation state through time. It must reuse Core for shared contracts, ECS for entities/components, Compute for operations over data and Runtime only when work coordination is needed. Simulation defines the meaning of a step, the timestep, the update order and error handling; it must not become another scheduler or a storage system.

- **Core:** shared types and errors, no simulation policy.
- **ECS:** entity lifecycle and state represented through components.
- **Compute:** explicit sequential/parallel processing of buffers and slices.
- **Runtime:** task coordination when an operation requires it; it does not define time semantics.
- **Simulation:** time configuration, step state, phase order and reproducible-update contracts.

The first version will be Rust-first, safe and small, with no external dependencies, global clock, I/O, hidden allocation or automatic parallel execution. `#![forbid(unsafe_code)]` is kept.

## Limits of the first version

The first API must allow building and advancing a simulation with explicit time, validating the configuration and testing reproducible results. It will not attempt to be a complete physics engine or impose a universal representation of positions, fluids, cells, rays or agents. Those domains may be added later as independent models or examples once their contracts and needs are clear.

Out of initial scope: implicit wall clock, automatic variable step, rendering, input/output, networking, rollback, a scenario DSL, a plugin system, a hidden scheduler, real-time guarantees and bit-for-bit determinism across architectures for floating-point operations.

## Implemented API

The API available in `dinah::simulation` includes `SimulationClock`, `SimulationTime`, `SimulationTimeError`, `Simulation<S>`, `SimulationStepError`, `SimulationEntityId`, `SimulationEntities<T>`, `SimulationEntitiesError`, `SimulationEntityBindings` and `SimulationEntityBindingError`. The following snippet only preserves the initial conceptual sketch; it is not an additional API or a signature to copy literally:

```rust
pub struct SimulationConfig {
    pub fixed_delta: Duration,
    pub max_steps_per_advance: NonZeroUsize,
}

pub struct SimulationTime {
    pub tick: u64,
    pub elapsed: Duration,
    pub delta: Duration,
}

pub enum SimulationError {
    ZeroDelta,
    StepLimitReached,
    TickOverflow,
    Update(/* preserved domain error */),
}

pub trait SimulationStep {
    type Error;

    fn update(&mut self, time: SimulationTime) -> Result<(), Self::Error>;
}
```

The implemented clock provides the temporal core of this proposal: `SimulationClock::new(delta)` rejects a zero delta and `advance()` increments a tick and the accumulated time in a checked manner. If the tick or duration overflows, it returns `SimulationTimeError` and leaves the clock intact. The policy for external time accumulation and maximum steps per advance is deferred; the clock reads no wall-clock time and runs no callbacks. `Simulation<S>::step` computes the next time on a copy of the clock, calls the synchronous callback once and only commits the clock after success. If the callback returns an error, the tick does not advance; domain mutations made before the error are not automatically rolled back. If the time computation fails, the callback is not even invoked.

## Simulated entities (point 113)

`SimulationEntities<T>` keeps entity states owned by Simulation in an ordered vector. `insert` returns a monotonic `SimulationEntityId`; `get`, `get_mut`, `remove`, `iter`, `len` and `is_empty` expose explicit operations. Iteration respects insertion order and `remove` preserves the relative order of the remaining entities. Removed IDs are not reused, and ID-space exhaustion is reported before modifying the collection. Lookup by ID and removal are linear in this first version; no constant-time performance or ECS storage is claimed.

These IDs are deliberately distinct from `ecs::Entity`: this collection allows defining the simulated-state contract first without duplicating or assuming the `World` lifecycle. Point 114 implements the explicit `SimulationEntityBindings` bridge: `bind` only accepts live entities in the supplied `World` and avoids assigning two IDs to the same handle; `resolve` re-checks the handle's liveness and rejects removed entities or stale generations; `unbind` removes only the association and never destroys the entity. The caller must consistently supply the same owning `World`, because ECS handles are local to each world. This bridge's IDs belong to their own sequence and are not `SimulationEntities<T>` IDs. The association takes no ownership of the ECS lifecycle and does not automatically synchronize `despawn`; the caller must manage `unbind` where appropriate. To process simulated states, `SimulationEntities::for_each_mut` delegates the mutable traversal to the sequential `compute::for_each` primitive, preserves insertion order and creates no workers or clock changes. A panic propagates and prior mutations are not rolled back.

## Contracts that must be defined

### Time and advancement

- The simulation step is expressed with an explicit duration; no global clock is read.
- The fixed timestep will be the reference deterministic route. Variable stepping, if needed, will be a later explicit capability.
- Every step receives a monotonic index and a validated delta; counter overflow must be detected, not silently wrapped.
- Steps-per-advance limits must be configurable and must not silently discard time.
- The phase order and the moment the tick increments must be documented and tested.

### Updates, errors and panics

- An update returns success or its domain error; a failure is not replaced by a default value.
- No state rollback is promised if an update fails after mutations. If atomicity is needed, it must be designed explicitly through temporary state or transactions.
- Panics are not silently converted into ordinary errors; Runtime's documented policy applies when its workers are used.
- A failed update does not silently advance the tick. Point 112 must fix the exact rule and cover it with tests.

### Determinism

- With the same configuration, initial state and explicit input sequence, updates must follow the same logical order.
- No temporal determinism or bit-for-bit equality of floating-point operations across architectures is promised.
- Iterations over unordered maps/sets cannot define the step's observable order.
- Random seeds, if introduced, must be explicit and serializable; there will be no implicit global randomness in the core.

### ECS and Compute integration

- ECS integration happens through explicit associations and must respect borrows, entity lifecycle and the limits of actually implemented queries.
- Compute integration is explicit: `for_each_mut` is sequential; an operation does not become parallel just because it processes many elements.
- ECS systems will not run in parallel until point 100 has safe access validation and structural isolation.
- The API will not require `Send + Sync` on all models if sequential execution does not need it.

## Proposed implementation sequence

1. **105 — API design:** completed in this document; initial signatures and limits defined.
2. **106–110 — Bounded domains (implemented):** `simulation::ascii_physics`, `simulation::fluid`, `simulation::cellular`, `simulation::raycast` and `simulation::swarm` contain initial models independent of the generic clock. Each has explicit limits and domain tests; they are not general-purpose engines. Implementing these modules does provide evidence for 106–110, but it does not imply physics capabilities, CFD, 3D raycasting or unbounded agent simulation. The concrete contracts and restrictions of each model are described in the "Bounded domains (points 106–110)" section at the end of this guide.
3. **111 — Timestep:** implemented as `SimulationClock`, with a non-zero fixed delta, tick, accumulated time, explicit advancement and checked overflow errors. Steps-per-call limits and the external time accumulator remain for later.
4. **112 — Deterministic update:** completed in the first version through a synchronous per-step callback, explicit time and clock commit only after success; domain state has no automatic rollback.
5. **113–115 — Entities and integration:** 113 creates the simulated-state collection; 114 adds the explicit bridge between simulated IDs and ECS handles; 115 integrates sequential mutable traversal with Compute, preserving order and without activating implicit parallelism.
6. **116–118 — Usage quality:** reproducible tests (116), an initial benchmark measured and archived on five platforms (117) and the runnable example `examples/simulation_basics.rs` (118).
7. **119 — Documentation:** this guide covers the implemented API, its limits, baseline measurements and a runnable example. Documentation will keep evolving as the module grows.

## Deterministic tests (point 116)

The `deterministic_replay_reproduces_tick_trace_and_entity_state` test runs the same scenario twice with a fixed delta and an explicit input sequence. Every tick updates entities in insertion order via `for_each_mut`; the trace records the tick and each ID/state pair. The two traces and final states must match exactly. The test reads no wall clock, uses no randomness sources and does not depend on worker completion order. This verifies logical reproducibility for this concrete sequential workload; it is not a guarantee of bit-for-bit determinism across architectures or of parallel execution.

## Benchmarks (point 117)

**Point 117 completed with real CI measurements** on commit `83cbcc3f3a36bd38ee661ae12075a0e27b654414`, workflow [dinah benchmarks, run 38484605568](https://github.com/lecodev-26/dinah/actions/runs/38484605568). The complete outputs and environments are archived in [`artifacts/benchmarks/83cbcc3f3a36bd38ee661ae12075a0e27b654414/run-38484605568/`](../artifacts/benchmarks/83cbcc3f3a36bd38ee661ae12075a0e27b654414/run-38484605568/), with results for Linux x86-64, Linux ARM64, Windows x86-64, macOS x86-64 and macOS ARM64.

`benches/simulation.rs` runs in release, uses nine samples and reports medians; each sample repeats the operation 32 times. It includes 256 `SimulationClock` advances (with construction), 256 `Simulation` steps with a small update (with construction), mutable traversal over collections of 64, 1,024 and 16,384 entities and 16 `get` lookups distributed over each collection. Reference table for the time cases and the 16,384-entity size:

| Platform | Clock (ns/advance) | Step (ns/step) | Mutable walk (ns/entity) | 16 linear searches (ns/search) |
| --- | ---: | ---: | ---: | ---: |
| Linux x86-64 | 7.801 | 12.547 | 0.368 | 2,437.000 |
| Linux ARM64 | 5.855 | 8.000 | 0.438 | 1,792.938 |
| Windows x86-64 | 7.836 | 12.242 | 0.346 | 2,446.062 |
| macOS x86-64 | 10.488 | 10.305 | 0.823 | 4,990.062 |
| macOS ARM64 | 2.395 | 6.012 | 0.281 | 1,533.875 |

These are observations from specific machines and builds, not guarantees or controlled comparisons between operating systems. Construction is included in the clock and step cases, so they must not be interpreted as the isolated per-tick cost in steady state. The search case measures the current linear lookup over 16 IDs spread across the collection; it is not an ECS query. This data establishes a baseline; it does not prove an improvement over a previous version or by itself justify changing the storage representation. The benchmark workflow runs the suite and archives the output and environment of the five runners.

## Runnable example (point 118)

`cargo run --example simulation_basics` shows a `Simulation<Vec<Particle>>` advancing three steps with a fixed 16 ms delta and updating position using the logical delta delivered to the callback. It then creates a `SimulationEntities<Particle>`, updates its states with `for_each_mut` and walks the results in insertion order. The example uses no wall clock, no implicit workers and does not automatically integrate simulated entities with ECS.

## Current criteria and limits

- The published API corresponds to the code in `src/simulation.rs`; the design avoids duplicating Core, ECS or Runtime responsibilities.
- `SimulationClock` rejects a zero delta and checks tick and duration overflow before mutating its state.
- `Simulation::step` invokes a synchronous callback with explicit time and commits the clock only after success. Domain state has no automatic rollback if the callback mutates and then fails.
- The replay tests compare traces and state for an explicit sequence, with no real clock or implicit randomness. They do not promise bit-for-bit determinism across architectures or parallel-execution safety.
- The benchmarks report size, configuration, environment and medians. They are microbenchmark baselines, not improvement proofs or universal predictions.
- `cargo run --example simulation_basics` uses the real API. Maximum steps per advance, the external time accumulator, pause policy, rollback/transactions, serialization and domain models remain outside the implemented scope.

## Deferred decisions

Still open: whether external time accumulation is allowed, the pause policy, state serialization, events, rollback/transactions of domain state, the concrete integration with each domain and the parallel strategy. They will be resolved with tests and real needs, not by adding preventive abstractions.

## Bounded domains (points 106–110)

The following modules are educational models independent of the generic clock. They include no external dependencies and are not presented as general-purpose engines:

- **106 — `simulation::ascii_physics`:** `AsciiPhysics` creates a finite grid, records obstacles and particles with integer velocity, advances each particle one cell per active axis, bounces off walls/obstacles and renders `#`, `o` and spaces. Particle IDs are insertion indices; there is no inter-particle collision resolution or continuous physical conservation.
- **107 — `simulation::fluid`:** `ScalarFluid` solves explicit 2D scalar diffusion with four neighbors, a double buffer and a no-flow boundary. The coefficient must belong to `[0, 0.25]`, the classic stability limit for this 2D discretization. It does not simulate pressure, velocity or full fluid dynamics; `total()` serves as an approximate conservation check with floating-point tolerance.
- **108 — `simulation::cellular`:** `CellularGrid` implements Conway's Game of Life with Moore neighborhood and synchronous update via a double buffer. Boundaries can be `Boundary::Dead` or `Boundary::Wrap`. There are no custom rules in this first API.
- **109 — `simulation::raycast`:** `Ray2` and `Aabb2` implement 2D interval (slab test) intersection. `RayHit::distance` is the `t` parameter of `origin + direction * t`, not a Euclidean distance unless the direction is normalized. An origin inside the AABB returns `t = 0`; a miss returns `None`. Non-finite inputs, zero directions and inverted boxes are rejected when constructing the types.
- **110 — `simulation::swarm`:** `Flock2D` averages position and velocity of agents within the neighborhood radius, applies a steering direction bounded by `max_speed` and computes all results from the same snapshot. It is a small flocking model, not a complete implementation of all Reynolds rules or a promise of bit-for-bit equality across architectures.

The modules use `f64` where the model is continuous; the tests check same-platform reproducibility, limits, patterns and relevant numeric properties. There is no implicit parallel execution. Points 106–110 were only closed after CI confirmed tests, Clippy and public documentation.
