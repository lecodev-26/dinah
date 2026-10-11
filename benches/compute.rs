use std::hint::black_box;
use std::time::{Duration, Instant};

use dinah::compute::for_each;
use wide::f32x4;

const SAMPLES: usize = 11;
const ITERATIONS_PER_SAMPLE: usize = 10;
const SIZES: &[usize] = &[64, 65, 1_024, 1_027, 16_384, 16_387, 262_144, 262_147];

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

fn transform_wide(values: &mut [f32]) {
    let multiplier = f32x4::splat(1.0001);
    let offset = f32x4::splat(0.25);
    let (chunks, remainder) = values.as_chunks_mut::<4>();
    for chunk in chunks {
        let lanes = f32x4::new(*chunk);
        let transformed = lanes * multiplier + offset;
        *chunk = transformed.to_array();
    }
    for value in remainder {
        *value = *value * 1.0001 + 0.25;
    }
}

fn assert_close(left: &[f32], right: &[f32]) {
    assert_eq!(left.len(), right.len());
    for (index, (left, right)) in left.iter().zip(right).enumerate() {
        let tolerance = 1.0e-5_f32 * left.abs().max(right.abs()).max(1.0);
        assert!(
            (left - right).abs() <= tolerance,
            "results differ at index {index}: {left} vs {right} (tolerance {tolerance})"
        );
    }
}

fn report(case: &str, elements: usize, samples: &[Duration]) {
    let mut values: Vec<_> = samples.iter().map(Duration::as_nanos).collect();
    values.sort_unstable();
    let median = values[values.len() / 2];
    println!(
        "case={case} elements={elements} samples={SAMPLES} iterations_per_sample={ITERATIONS_PER_SAMPLE} median_op_ns={median} ns_per_element={:.3} elements_per_second={:.0}",
        median as f64 / elements.max(1) as f64,
        if median == 0 { 0.0 } else { elements as f64 * 1_000_000_000.0 / median as f64 },
    );
}

fn main() {
    println!(
        "dinah compute SIMD investigation; build=release; statistic=median; samples={SAMPLES}; iterations_per_sample={ITERATIONS_PER_SAMPLE}"
    );
    println!(
        "Compares scalar loops with a safe wide::f32x4 candidate; SIMD speedup is not assumed."
    );
    println!("target={}", std::env::consts::ARCH);

    for &size in SIZES {
        let initial: Vec<f32> = (0..size).map(|i| (i as f32 * 0.25) - 10.0).collect();
        let mut direct = initial.clone();
        let mut api = initial.clone();
        let mut vector = initial;

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

        let vector_samples = measure(|| {
            transform_wide(black_box(&mut vector));
            black_box(&vector);
        });

        assert_eq!(
            direct, api,
            "both scalar loops must implement the same transform"
        );
        assert_close(&direct, &vector);
        report("direct_scalar_transform", size, &direct_samples);
        report("compute_for_each_transform", size, &api_samples);
        report("wide_f32x4_candidate_transform", size, &vector_samples);
    }
}
