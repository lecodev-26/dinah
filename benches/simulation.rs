use std::hint::black_box;
use std::time::{Duration, Instant};

use dinah::simulation::{Simulation, SimulationClock, SimulationEntities};

const SAMPLES: usize = 9;
const ITERATIONS_PER_SAMPLE: usize = 32;
const SIZES: &[usize] = &[64, 1_024, 16_384];
const LOOKUPS_PER_SAMPLE: usize = 16;

fn measure<F: FnMut()>(mut operation: F) -> Vec<Duration> {
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        for _ in 0..ITERATIONS_PER_SAMPLE {
            operation();
        }
        samples.push(start.elapsed() / ITERATIONS_PER_SAMPLE as u32);
    }
    samples
}

fn report(case: &str, operations: usize, samples: &[Duration]) {
    let mut values: Vec<_> = samples.iter().map(Duration::as_nanos).collect();
    values.sort_unstable();
    let median = values[values.len() / 2];
    let min = values[0];
    let max = values[values.len() - 1];
    println!(
        "case={case} operations={operations} samples={SAMPLES} iterations_per_sample={ITERATIONS_PER_SAMPLE} min_op_ns={min} median_op_ns={median} max_op_ns={max} ns_per_operation={:.3}",
        median as f64 / operations.max(1) as f64,
    );
}

fn main() {
    println!("dinah Simulation benchmark; build=release; statistic=median; samples={SAMPLES}; iterations_per_sample={ITERATIONS_PER_SAMPLE}");
    println!(
        "target={}; results are machine-specific observations, not guarantees",
        std::env::consts::ARCH
    );

    let clock_samples = measure(|| {
        let mut clock = SimulationClock::new(Duration::from_millis(16)).expect("non-zero delta");
        for _ in 0..256 {
            black_box(
                clock
                    .advance()
                    .expect("tick and elapsed time do not overflow"),
            );
        }
        black_box((clock.tick(), clock.elapsed()));
    });
    report(
        "simulation_clock_256_advances_including_setup",
        256,
        &clock_samples,
    );

    let step_samples = measure(|| {
        let mut simulation =
            Simulation::new(0_u64, Duration::from_millis(16)).expect("non-zero delta");
        for _ in 0..256 {
            simulation
                .step(|state, time| {
                    *state = state.wrapping_add(time.tick);
                    Ok::<(), ()>(())
                })
                .expect("update succeeds");
        }
        black_box((simulation.state(), simulation.clock().tick()));
    });
    report(
        "simulation_step_256_updates_including_setup",
        256,
        &step_samples,
    );

    for &size in SIZES {
        let mut entities = SimulationEntities::new();
        for value in 0..size {
            entities
                .insert(value as u64)
                .expect("ID space is sufficient for benchmark size");
        }
        let update_samples = measure(|| {
            entities.for_each_mut(|id, state| {
                *state = state.wrapping_add(id.get());
            });
            black_box(entities.len());
        });
        report("simulation_entities_for_each_mut", size, &update_samples);

        let ids: Vec<_> = entities
            .iter()
            .step_by((size / LOOKUPS_PER_SAMPLE).max(1))
            .take(LOOKUPS_PER_SAMPLE)
            .map(|(id, _)| id)
            .collect();
        let lookup_samples = measure(|| {
            let checksum = ids.iter().fold(0_u64, |sum, &id| {
                sum.wrapping_add(*black_box(
                    entities.get(id).expect("sampled entity remains present"),
                ))
            });
            black_box(checksum);
        });
        report(
            "simulation_entities_16_spread_get_lookups_linear_search",
            ids.len(),
            &lookup_samples,
        );
    }
}
