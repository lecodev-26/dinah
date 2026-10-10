use std::hint::black_box;
use std::time::{Duration, Instant};

use dinah::memory::{Buffer, SoaBuffer};
use dinah::runtime::Shared;

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
    println!("dinah memory benchmark; build=release; statistic=median; samples={SAMPLES}");
    println!("Note: results are machine-specific observations, not performance guarantees.");

    for &size in SIZES {
        let push = measure(|| {
            let mut buffer =
                Buffer::try_with_capacity(size).expect("capacity reservation succeeds");
            for value in 0..size {
                buffer
                    .try_push(black_box(value))
                    .expect("buffer capacity matches workload");
            }
            black_box(buffer.as_slice());
        });
        report("buffer_push_preallocated", size, &push);

        let values: Vec<usize> = (0..size).collect();
        let extend = measure(|| {
            let mut buffer =
                Buffer::try_with_capacity(size).expect("capacity reservation succeeds");
            buffer
                .try_extend_from_slice(black_box(&values))
                .expect("slice fits buffer");
            black_box(buffer.as_slice());
        });
        report("buffer_bulk_extend", size, &extend);

        let soa_push = measure(|| {
            let mut rows =
                SoaBuffer::try_with_capacity(size).expect("capacity reservation succeeds");
            for value in 0..size {
                rows.try_push(black_box(value), black_box(value.wrapping_add(1)))
                    .expect("row capacity matches workload");
            }
            black_box(rows.as_slices());
        });
        report("soa_push_rows", size, &soa_push);

        let first: Vec<usize> = (0..size).collect();
        let second: Vec<usize> = (0..size).map(|value| value.wrapping_add(1)).collect();
        let soa_extend = measure(|| {
            let mut rows =
                SoaBuffer::try_with_capacity(size).expect("capacity reservation succeeds");
            rows.try_extend_from_slices(black_box(&first), black_box(&second))
                .expect("both columns fit buffer");
            black_box(rows.as_slices());
        });
        report("soa_bulk_extend", size, &soa_extend);

        let soa_scan = {
            let mut rows =
                SoaBuffer::try_with_capacity(size).expect("capacity reservation succeeds");
            for value in 0..size {
                rows.try_push(value, value.wrapping_add(1))
                    .expect("row capacity matches workload");
            }
            measure(|| {
                let (first, second) = rows.as_slices();
                let checksum = first
                    .iter()
                    .zip(second)
                    .fold(0usize, |sum, (a, b)| sum.wrapping_add(*a ^ *b));
                black_box(checksum);
            })
        };
        report("soa_scan_two_columns", size, &soa_scan);
    }

    let shared = Shared::new(0usize);
    let lock_operations = 10_000usize;
    let lock_samples = measure(|| {
        for _ in 0..lock_operations {
            *shared.lock().expect("shared lock remains healthy") += 1;
        }
        black_box(*shared.lock().expect("shared lock remains healthy"));
    });
    report("shared_mutex_lock_update", lock_operations, &lock_samples);
}
