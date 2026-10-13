use std::hint::black_box;
use std::time::{Duration, Instant};

use dinah::ecs::World;

const SAMPLES: usize = 7;
const SIZES: &[usize] = &[64, 1_024, 16_384];

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
    let count = operations.max(1) as f64;
    println!(
        "case={name} operations={operations} samples={SAMPLES} median_total_ns={median} ns_per_operation={:.3} operations_per_second={:.0}",
        median as f64 / count,
        if median == 0 { 0.0 } else { count * 1_000_000_000.0 / median as f64 },
    );
}

fn main() {
    println!("dinah ECS benchmark; build=release; statistic=median; samples={SAMPLES}");
    println!("Scope: public World entity lifecycle only; component storage and queries are not integrated yet.");
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
    }
}
