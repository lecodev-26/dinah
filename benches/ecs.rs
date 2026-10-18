use std::hint::black_box;
use std::time::{Duration, Instant};

use dinah::ecs::World;

const SAMPLES: usize = 7;
const SIZES: &[usize] = &[64, 1_024, 16_384];

#[derive(Clone, Copy)]
struct Position(u64);

fn measure<F: FnMut()>(mut operation: F) -> Vec<Duration> {
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        operation();
        samples.push(start.elapsed());
    }
    samples
}

fn report(name: &str, operations: usize, samples: &[Duration]) {
    let mut nanos: Vec<_> = samples.iter().map(Duration::as_nanos).collect();
    nanos.sort_unstable();
    let median = nanos[nanos.len() / 2];
    let min = nanos[0];
    let max = nanos[nanos.len() - 1];
    let count = operations.max(1) as f64;
    println!(
        "case={name} operations={operations} samples={SAMPLES} min_total_ns={min} median_total_ns={median} max_total_ns={max} ns_per_operation={:.3} operations_per_second={:.0}",
        median as f64 / count,
        if median == 0 { 0.0 } else { count * 1_000_000_000.0 / median as f64 },
    );
}

fn populated_world(size: usize, stride: usize) -> (World, Vec<dinah::ecs::Entity>) {
    let mut world = World::new();
    let mut selected = Vec::new();
    for index in 0..size {
        let entity = world.spawn().expect("entity allocation succeeds");
        if index % stride == 0 {
            world
                .insert(entity, Position(index as u64))
                .expect("live entity accepts component");
            selected.push(entity);
        }
    }
    (world, selected)
}

fn main() {
    println!("dinah ECS benchmark; build=release; statistic=median; samples={SAMPLES}");
    println!("Scope: public World entity lifecycle and individual component access; queries and public component iteration are not implemented.");
    println!("Results are machine-specific observations, not performance guarantees.");

    for &size in SIZES {
        let spawn = measure(|| {
            let mut world = World::new();
            for _ in 0..size {
                black_box(world.spawn().expect("entity allocation succeeds"));
            }
            black_box(world.len());
        });
        report("world_spawn_fresh", size, &spawn);

        let despawn = measure(|| {
            let mut world = World::new();
            let entities: Vec<_> = (0..size)
                .map(|_| world.spawn().expect("entity allocation succeeds"))
                .collect();
            for entity in entities {
                world.despawn(black_box(entity)).expect("entity is alive");
            }
            black_box(world.is_empty());
        });
        report("world_despawn_after_spawn", size, &despawn);

        let reuse = measure(|| {
            let mut world = World::new();
            let entities: Vec<_> = (0..size)
                .map(|_| world.spawn().expect("entity allocation succeeds"))
                .collect();
            for entity in entities {
                world.despawn(entity).expect("entity is alive");
            }
            for _ in 0..size {
                black_box(world.spawn().expect("free slot is reusable"));
            }
            black_box(world.len());
        });
        report("world_spawn_despawn_reuse", size * 3, &reuse);

        let lookup = {
            let mut world = World::new();
            let entities: Vec<_> = (0..size)
                .map(|_| world.spawn().expect("entity allocation succeeds"))
                .collect();
            measure(|| {
                let live = entities
                    .iter()
                    .filter(|&&entity| world.contains(black_box(entity)))
                    .count();
                black_box(live);
            })
        };
        report("world_contains_scan", size, &lookup);

        for (label, stride) in [("dense", 1_usize), ("sparse", 16_usize)] {
            let (world, entities) = populated_world(size, stride);
            let component_reads = measure(|| {
                let checksum = entities.iter().fold(0_u64, |sum, &entity| {
                    sum.wrapping_add(
                        black_box(world.get::<Position>(black_box(entity)))
                            .expect("selected entity has Position")
                            .0,
                    )
                });
                black_box(checksum);
            });
            report(
                if label == "dense" {
                    "world_component_get_dense"
                } else {
                    "world_component_get_sparse"
                },
                entities.len(),
                &component_reads,
            );
        }

        let stale_lookup = {
            let mut world = World::new();
            let old = world.spawn().expect("entity allocation succeeds");
            world
                .insert(old, Position(7))
                .expect("component insertion succeeds");
            world.despawn(old).expect("entity is alive");
            let current = world.spawn().expect("free slot is reusable");
            world
                .insert(current, Position(11))
                .expect("component insertion succeeds");
            measure(|| {
                let misses = (0..size)
                    .filter(|_| black_box(world.get::<Position>(black_box(old))).is_none())
                    .count();
                black_box(misses);
            })
        };
        report("world_component_get_stale_generation", size, &stale_lookup);

        let insert_replace_remove = measure(|| {
            let mut world = World::new();
            let entity = world.spawn().expect("entity allocation succeeds");
            for value in 0..size {
                let previous = world
                    .insert(entity, Position(value as u64))
                    .expect("component insertion succeeds");
                black_box(previous);
            }
            black_box(world.remove::<Position>(entity));
        });
        report(
            "world_component_insert_replace_remove",
            size + 1,
            &insert_replace_remove,
        );
    }
}
