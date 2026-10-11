use std::hint::black_box;
use std::time::{Duration, Instant};

use dinah::compute::for_each;

const SAMPLES: usize = 11;
const SIZES: &[usize] = &[64, 1_024, 16_384, 262_144];

fn measure<F: FnMut()>(mut operation: F) -> Vec<Duration> {
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        operation();
        samples.push(start.elapsed());
    }
    samples
}

fn report(case: &str, elements: usize, samples: &[Duration]) {
    let mut values: Vec<_> = samples.iter().map(Duration::as_nanos).collect();
    values.sort_unstable();
    let median = values[values.len() / 2];
    println!(
        "case={case} elements={elements} samples={SAMPLES} median_total_ns={median} ns_per_element={:.3} elements_per_second={:.0}",
        median as f64 / elements.max(1) as f64,
        if median == 0 { 0.0 } else { elements as f64 * 1_000_000_000.0 / median as f64 },
    );
}

fn main() {
    println!(
        "dinah compute SIMD investigation; build=release; statistic=median; samples={SAMPLES}"
    );
    println!("Compares equivalent safe in-place scalar loops; no manual SIMD is claimed.");
    println!("target={}", std::env::consts::ARCH);

    for &size in SIZES {
        let initial: Vec<f32> = (0..size).map(|i| (i as f32 * 0.25) - 10.0).collect();
        let mut direct = initial.clone();
        let mut api = initial;

        let direct_samples = measure(|| {
            for value in black_box(&mut direct).iter_mut() {
                *value = *value * 1.0001 + 0.25;
            }
            black_box(&direct);
        });
        let api_samples = measure(|| {
            for_each(black_box(&mut api), |value| {
                *value = *value * 1.0001 + 0.25;
            });
            black_box(&api);
        });

        assert_eq!(direct, api, "both loops must implement the same transform");
        report("direct_scalar_transform", size, &direct_samples);
        report("compute_for_each_transform", size, &api_samples);
    }
}
