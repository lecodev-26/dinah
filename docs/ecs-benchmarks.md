# DINAH ECS benchmarks

The `ecs` release benchmark records the cost of the currently public entity-lifecycle API. It runs seven samples per case and reports the median total duration, nanoseconds per operation, and derived operations per second for workloads of 64, 1,024, and 16,384 entities.

Cases:

- `world_spawn_fresh`: create a fresh `World` and allocate the requested number of entities.
- `world_despawn_after_spawn`: allocate the workload, then destroy each entity.
- `world_spawn_despawn_reuse`: allocate, destroy, then allocate again to exercise free-slot reuse and generation advancement.
- `world_contains_scan`: scan a prebuilt set of live handles and count successful `contains` checks.

The benchmark intentionally uses only public API. It does **not** measure component insertion, component iteration, queries, or structural changes involving components: `ComponentStorage` is still internal and is not connected to `World`. These results must not be interpreted as full ECS workload performance or as proof that the ECS is faster than another implementation.

The benchmark workflow records results on Linux x86-64, Linux ARM64, Windows x86-64, macOS x86-64, and macOS ARM64. Compare only results from comparable environments; use measurements to identify follow-up questions, not to infer universal speedups. The cases include world construction and allocation where described, so their timing is end-to-end for that scenario rather than a pure isolated instruction cost.
