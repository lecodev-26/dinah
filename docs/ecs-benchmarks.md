# DINAH ECS benchmarks

The `ecs` release benchmark records the cost of the currently public entity-lifecycle API. It runs seven samples per case and reports the median total duration, nanoseconds per operation, and derived operations per second for workloads of 64, 1,024, and 16,384 entities.

Cases:

- `world_spawn_fresh`: create a fresh `World` and allocate the requested number of entities.
- `world_despawn_after_spawn`: allocate the workload, then destroy each entity.
- `world_spawn_despawn_reuse`: allocate, destroy, then allocate again to exercise free-slot reuse and generation advancement.
- `world_contains_scan`: scan a prebuilt set of live handles and count successful `contains` checks.

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
