use std::hint::black_box;
use std::time::{Duration, Instant};

use dinah::memory::{Buffer, SoaBuffer};

#[derive(Clone, Copy)]
struct DataRow {
    first: usize,
    second: usize,
}
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
    let min = nanos[0];
    let max = nanos[nanos.len() - 1];
    let count = operations.max(1) as f64;
    println!(
        "case={name} operations={operations} samples={SAMPLES} min_total_ns={min} median_total_ns={median} max_total_ns={max} ns_per_operation={:.3} operations_per_second={:.0}",
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

        let first: Vec<usize> = (0..size).collect();
        let second: Vec<usize> = (0..size).map(|value| value.wrapping_add(1)).collect();
        let soa_push = measure(|| {
            let mut rows =
                SoaBuffer::try_with_capacity(size).expect("capacity reservation succeeds");
            for (&a, &b) in black_box(&first).iter().zip(black_box(&second)) {
                rows.try_push(black_box(a), black_box(b))
                    .expect("row capacity matches workload");
            }
            black_box(rows.as_slices());
        });
        report("soa_push_rows", size, &soa_push);

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

        // Compare data layouts on identical prebuilt values. Construction and
        // conversion costs are intentionally excluded from these access-only
        // cases and should be measured separately for a concrete workload.
        let aos: Vec<DataRow> = (0..size)
            .map(|first| DataRow {
                first,
                second: first.wrapping_add(1),
            })
            .collect();
        let soa = {
            let mut rows =
                SoaBuffer::try_with_capacity(size).expect("capacity reservation succeeds");
            let first: Vec<usize> = (0..size).collect();
            let second: Vec<usize> = (0..size).map(|value| value.wrapping_add(1)).collect();
            rows.try_extend_from_slices(&first, &second)
                .expect("both columns fit buffer");
            rows
        };

        let aos_one_field = measure(|| {
            let checksum = black_box(&aos)
                .iter()
                .fold(0usize, |sum, row| sum.wrapping_add(black_box(row.first)));
            black_box(checksum);
        });
        report("layout_aos_scan_one_field", size, &aos_one_field);

        let soa_one_field = measure(|| {
            let (first, _) = soa.as_slices();
            let checksum = black_box(first)
                .iter()
                .fold(0usize, |sum, value| sum.wrapping_add(black_box(*value)));
            black_box(checksum);
        });
        report("layout_soa_scan_one_field", size, &soa_one_field);

        let aos_both_fields = measure(|| {
            let checksum = black_box(&aos).iter().fold(0usize, |sum, row| {
                sum.wrapping_add(black_box(row.first).wrapping_add(black_box(row.second)))
            });
            black_box(checksum);
        });
        report("layout_aos_scan_both_fields", size, &aos_both_fields);

        let soa_both_fields = measure(|| {
            let (first, second) = soa.as_slices();
            let checksum = black_box(first)
                .iter()
                .zip(black_box(second))
                .fold(0usize, |sum, (a, b)| {
                    sum.wrapping_add(black_box(*a).wrapping_add(black_box(*b)))
                });
            black_box(checksum);
        });
        report("layout_soa_scan_both_fields", size, &soa_both_fields);
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
