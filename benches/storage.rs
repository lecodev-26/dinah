use dinah::storage::{MemoryStore, Storage};
use std::hint::black_box;
use std::time::{Duration, Instant};

const SAMPLES: usize = 9;
const ITERATIONS: usize = 1_000;
fn measure(mut operation: impl FnMut()) -> Vec<Duration> {
    (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                operation();
            }
            start.elapsed() / ITERATIONS as u32
        })
        .collect()
}
fn report(name: &str, samples: &[Duration]) {
    let mut nanos: Vec<_> = samples.iter().map(Duration::as_nanos).collect();
    nanos.sort_unstable();
    println!(
        "case={name} iterations={ITERATIONS} samples={SAMPLES} median_op_ns={}",
        nanos[nanos.len() / 2]
    );
}
fn main() {
    println!("dinah Storage benchmark; build=release; statistic=median; architecture={}; observations are machine-specific", std::env::consts::ARCH);
    report(
        "memory_put_get_256_keys",
        &measure(|| {
            let mut store = MemoryStore::new();
            for i in 0..256_u16 {
                store.put(&i.to_le_bytes(), &i.to_le_bytes()).unwrap();
            }
            for i in 0..256_u16 {
                black_box(store.get(&i.to_le_bytes()));
            }
        }),
    );
    let mut store = MemoryStore::new();
    for i in (0..256_u16).rev() {
        store.put(&i.to_le_bytes(), &i.to_le_bytes()).unwrap();
    }
    report(
        "memory_ordered_scan_256_keys",
        &measure(|| {
            black_box(store.iter().count());
        }),
    );
}
