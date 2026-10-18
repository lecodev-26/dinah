use dinah::network::latency::LatencyEstimator;
use dinah::network::reliability::ReliableOutbox;
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
    let min = nanos[0];
    let median = nanos[nanos.len() / 2];
    let max = nanos[nanos.len() - 1];
    println!(
        "case={name} iterations={ITERATIONS} samples={SAMPLES} min_op_ns={min} median_op_ns={median} max_op_ns={max}",
    );
}
fn main() {
    println!("dinah Network benchmark; build=release; statistic=median; architecture={}; observations are machine-specific", std::env::consts::ARCH);
    report(
        "latency_estimator_128_samples",
        &measure(|| {
            let mut estimator = LatencyEstimator::new();
            for sample in 0..128 {
                estimator.record_sample(Duration::from_micros(500 + sample));
            }
            black_box((
                estimator.smoothed_rtt(),
                estimator.minimum_rtt(),
                estimator.maximum_rtt(),
            ));
        }),
    );
    report(
        "reliable_outbox_enqueue_ack_64",
        &measure(|| {
            let mut outbox = ReliableOutbox::new(64).expect("positive capacity");
            for value in 0..64_u64 {
                black_box(outbox.enqueue(value, 0).expect("capacity is available"));
            }
            black_box(outbox.acknowledge(63));
        }),
    );
}
