use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use dinah::runtime::{Executor, Scheduler, TaskQueue};

const SAMPLES: usize = 7;
const TASK_SIZES: &[usize] = &[1, 16, 64, 256];

fn noop_task() -> Result<u64, &'static str> {
    Ok(black_box(1))
}

fn report(name: &str, task_count: usize, samples: &[Duration]) {
    let mut nanos: Vec<u128> = samples.iter().map(Duration::as_nanos).collect();
    nanos.sort_unstable();
    let median_ns = nanos[nanos.len() / 2];
    let operations = task_count.max(1) as u128;
    let ns_per_task = median_ns as f64 / operations as f64;
    let tasks_per_second = if median_ns == 0 {
        0.0
    } else {
        (operations as f64 * 1_000_000_000.0) / median_ns as f64
    };
    println!(
        "case={name} tasks={task_count} samples={SAMPLES} median_total_ns={median_ns} ns_per_task={ns_per_task:.3} tasks_per_second={tasks_per_second:.0}"
    );
}

fn measure<F>(mut operation: F) -> Vec<Duration>
where
    F: FnMut(),
{
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        operation();
        samples.push(start.elapsed());
    }
    samples
}

fn main() {
    println!("dinah runtime benchmark; build=release; statistic=median; samples={SAMPLES}");
    println!("Note: results are machine-specific observations, not performance guarantees.");

    let executor = Executor::new();
    let direct_iterations = 100_000usize;
    let direct_samples = measure(|| {
        for _ in 0..direct_iterations {
            black_box(
                executor
                    .execute(noop_task)
                    .expect("benchmark task succeeds"),
            );
        }
    });
    report("executor_noop", direct_iterations, &direct_samples);

    let scheduler = Scheduler::new();
    let immediate_iterations = 100_000usize;
    let immediate_samples = measure(|| {
        for _ in 0..immediate_iterations {
            black_box(
                scheduler
                    .schedule(noop_task)
                    .expect("immediate scheduler accepts work")
                    .expect("benchmark task succeeds"),
            );
        }
    });
    report(
        "scheduler_immediate_noop",
        immediate_iterations,
        &immediate_samples,
    );

    for &task_count in TASK_SIZES {
        let sequential_samples = measure(|| {
            let capacity = NonZeroUsize::new(task_count).expect("task size is nonzero");
            let mut queue = TaskQueue::new(capacity);
            for _ in 0..task_count {
                queue
                    .try_push(noop_task as fn() -> Result<u64, &'static str>)
                    .expect("queue capacity matches task count");
            }
            let results = scheduler.run_until_empty(&mut queue);
            black_box(results);
            assert!(queue.is_empty());
        });
        report(
            "scheduler_queue_sequential",
            task_count,
            &sequential_samples,
        );
    }

    for &task_count in &[16usize, 64, 256] {
        for &worker_count in &[2usize, 4] {
            let parallel_samples = measure(|| {
                let capacity = NonZeroUsize::new(task_count).expect("task size is nonzero");
                let mut queue = TaskQueue::new(capacity);
                for _ in 0..task_count {
                    queue
                        .try_push(noop_task as fn() -> Result<u64, &'static str>)
                        .expect("queue capacity matches task count");
                }
                let workers = NonZeroUsize::new(worker_count).expect("worker count is nonzero");
                let results = scheduler.run_parallel(&mut queue, workers);
                black_box(results);
                assert!(queue.is_empty());
            });
            report(
                if worker_count == 2 {
                    "scheduler_queue_parallel_2_workers"
                } else {
                    "scheduler_queue_parallel_4_workers"
                },
                task_count,
                &parallel_samples,
            );
        }
    }
}
