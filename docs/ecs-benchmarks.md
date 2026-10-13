# DINAH ECS benchmarks

The `ecs` release benchmark records the cost of the currently public entity-lifecycle API and individual component access. It runs seven samples per case and reports the median total duration, nanoseconds per operation, and derived operations per second for workloads of 64, 1,024, and 16,384 entities.

Cases:

- `world_spawn_fresh`: create a fresh `World` and allocate the requested number of entities.
- `world_despawn_after_spawn`: allocate the workload, then destroy each entity.
- `world_spawn_despawn_reuse`: allocate, destroy, then allocate again to exercise free-slot reuse and generation advancement.
- `world_contains_scan`: scan a prebuilt set of live handles and count successful `contains` checks.
- `world_component_get_dense` / `world_component_get_sparse`: repeated typed `World::get` lookups for all entities or every sixteenth entity index with a component; the checksum consumes returned values.
- `world_component_get_stale_generation`: repeatedly verify that a stale handle cannot retrieve a component after its entity index is reused.
- `world_component_insert_replace_remove`: create one entity, insert then repeatedly replace one component, and remove it; this is an end-to-end structural workload, not an isolated insert latency.

The benchmark intentionally uses only public API. It does **not** measure component insertion, component iteration, queries, or structural changes involving components: `ComponentStorage` is still internal and is not connected to `World`. These results must not be interpreted as full ECS workload performance or as proof that the ECS is faster than another implementation.

The benchmark workflow records results on Linux x86-64, Linux ARM64, Windows x86-64, macOS x86-64, and macOS ARM64. Compare only results from comparable environments; use measurements to identify follow-up questions, not to infer universal speedups. The cases include world construction and allocation where described, so their timing is end-to-end for that scenario rather than a pure isolated instruction cost.


## First cross-platform baseline (commit `592b914`)

The first five-runner release run completed successfully: [benchmark workflow run 38476538971](https://github.com/lecodev-26/dinah/actions/runs/38476538971). At 16,384 entities, the median nanoseconds per operation reported by each runner were:

| Runner | Fresh spawn | Despawn after spawn | Spawn/despawn/reuse (per operation across 3 phases) | `contains` scan |
| --- | ---: | ---: | ---: | ---: |
| Linux x86-64 | 8.159 | 22.251 | 10.081 | 1.774 |
| Linux ARM64 | 3.192 | 10.050 | 4.523 | 0.916 |
| Windows x86-64 | 8.124 | 15.442 | 7.686 | 1.160 |
| macOS x86-64 | 8.750 | 11.595 | 6.041 | 1.560 |
| macOS ARM64 | 2.286 | 4.514 | 2.222 | 0.692 |

These are the exact reported per-operation medians for each runner and this workload; they are not directly comparable hardware benchmarks unless runner CPU, compiler, and environment are controlled. In particular, these results are not evidence of a universal platform ranking or a speedup against another ECS. The baseline confirms that the new benchmark executes on all five configured runners and provides reproducible starting data for later changes. Component and query performance remain unmeasured.


## Memory-access review (roadmap point 103, in progress)

The current implementation exposes individual typed component access through `World`, so the ECS benchmark now includes dense and sparse-index `get`, stale-generation misses, and an end-to-end insert/replace/remove case. The `memory` benchmark still measures generic `Buffer`/`SoaBuffer` operations and synthetic AoS/SoA scans. No public iteration or multi-component query API exists yet, so those workloads cannot honestly be measured through a consumer-facing interface.

The storage implementation already uses contiguous `entities` and `components` columns, a sparse index from entity index to row, generation validation on lookup, and `swap_remove` for deletion. These are design properties, not proof of cache-locality or speed. The sparse vector's memory footprint also depends on the largest entity index, so benchmarks must include sparse/dense entity-index distributions rather than only dense sequential handles.

**Decision for point 103:** do not change the representation or add speculative caching/prefetching yet. The current benchmark now covers: (1) repeated `get` for dense and sparse entity indices; (2) stale-generation misses; and (3) combined insert/replace/remove. The remaining required measurements are (4) sequential iteration over one component; (5) iteration over two component types with different overlap rates; and (6) end-to-end setup cost separated from steady-state access. Iteration and multi-component cases require the corresponding safe public API first. Use equivalent data and checksums, multiple sizes and repeated runs, and record allocations/memory footprint where practical. Compare against a simple baseline only if it models the same semantics. A representation change is justified only by a repeatable improvement in a representative workload without weakening generation checks, alignment invariants, allocation-failure behavior, or safe borrowing.

Until those cases can run through a real consumer API, point 103 remains **in progress**; no memory-access optimization is claimed.
