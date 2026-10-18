use std::hint::black_box;
use std::num::NonZeroUsize;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use dinah::runtime::{Executor, Scheduler, TaskQueue};

const SAMPLES: usize = 7;
const TASK_SIZES: &[usize] = &[1, 16, 64, 256];

enum PoolMessage {
    Run(Box<dyn FnOnce() + Send + 'static>),
    Shutdown,
}

/// Benchmark-only persistent pool prototype; not part of DINAH's public API.
struct PersistentPool {
    senders: Vec<mpsc::Sender<PoolMessage>>,
    workers: Vec<JoinHandle<()>>,
}

impl PersistentPool {
    fn new(worker_count: usize) -> Self {
        let mut senders = Vec::with_capacity(worker_count);
        let mut workers = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let (sender, receiver) = mpsc::channel::<PoolMessage>();
            senders.push(sender);
            workers.push(thread::spawn(move || {
                while let Ok(PoolMessage::Run(job)) = receiver.recv() {
                    job();
                }
            }));
        }
        Self { senders, workers }
    }

    fn run_tasks(
        &self,
        task_count: usize,
        worker_count: usize,
        task: fn() -> Result<u64, &'static str>,
    ) {
        let (done_sender, done_receiver) = mpsc::channel();
        let mut completed = 0;
        while completed < task_count {
            // Match the scheduler's batch assignment and barrier: each worker has
            // a dedicated channel, receives at most one task in this batch, and
            // the next batch is not sent until every task in this batch completes.
            let batch_size = worker_count.min(task_count - completed);
            for worker_index in 0..batch_size {
                let done_sender = done_sender.clone();
                self.senders[worker_index]
                    .send(PoolMessage::Run(Box::new(move || {
                        black_box(task().expect("benchmark pool task succeeds"));
                        done_sender
                            .send(())
                            .expect("benchmark completion receiver exists");
                    })))
                    .expect("benchmark pool is accepting work");
            }
            for _ in 0..batch_size {
                done_receiver
                    .recv()
                    .expect("every benchmark pool task completes");
            }
            completed += batch_size;
        }
        drop(done_sender);
    }
}

impl Drop for PersistentPool {
    fn drop(&mut self) {
        for sender in &self.senders {
            let _ = sender.send(PoolMessage::Shutdown);
        }
        for worker in self.workers.drain(..) {
            worker.join().expect("benchmark pool worker exits cleanly");
        }
    }
}

fn noop_task() -> Result<u64, &'static str> {
    Ok(black_box(1))
}

fn cpu_task() -> Result<u64, &'static str> {
    let mut value = black_box(1_u64);
    for _ in 0..1_000_000 {
        value = value
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
    }
    Ok(black_box(value))
}

fn report(name: &str, task_count: usize, samples: &[Duration]) {
    let mut nanos: Vec<u128> = samples.iter().map(Duration::as_nanos).collect();
    nanos.sort_unstable();
    let median_ns = nanos[nanos.len() / 2];
    let min_ns = nanos[0];
    let max_ns = nanos[nanos.len() - 1];
    let operations = task_count.max(1) as u128;
    let ns_per_task = median_ns as f64 / operations as f64;
    let tasks_per_second = if median_ns == 0 {
        0.0
    } else {
        (operations as f64 * 1_000_000_000.0) / median_ns as f64
    };
    println!(
        "case={name} tasks={task_count} samples={SAMPLES} min_total_ns={min_ns} median_total_ns={median_ns} max_total_ns={max_ns} ns_per_task={ns_per_task:.3} tasks_per_second={tasks_per_second:.0}"
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

    // This pool is only a comparison prototype. Its workers are created once
    // before measurement and reused across samples; startup cost is reported
    // separately so reuse is not compared unfairly with per-call thread startup.
    for &worker_count in &[2usize, 4] {
        let startup_samples = measure(|| {
            let pool = PersistentPool::new(worker_count);
            black_box(&pool);
            drop(pool);
        });
        report(
            if worker_count == 2 {
                "persistent_pool_startup_shutdown_2_workers"
            } else {
                "persistent_pool_startup_shutdown_4_workers"
            },
            1,
            &startup_samples,
        );
    }

    for &task_count in &[16usize, 64, 256] {
        let sequential_cpu_samples = measure(|| {
            let capacity = NonZeroUsize::new(task_count).expect("task size is nonzero");
            let mut queue = TaskQueue::new(capacity);
            for _ in 0..task_count {
                queue
                    .try_push(cpu_task as fn() -> Result<u64, &'static str>)
                    .expect("queue capacity matches task count");
            }
            let results = scheduler.run_until_empty(&mut queue);
            black_box(results);
            assert!(queue.is_empty());
        });
        report(
            "scheduler_queue_sequential_cpu_1m",
            task_count,
            &sequential_cpu_samples,
        );

        for &worker_count in &[2usize, 4] {
            let parallel_noop_samples = measure(|| {
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
                    "scheduler_queue_parallel_noop_2_workers"
                } else {
                    "scheduler_queue_parallel_noop_4_workers"
                },
                task_count,
                &parallel_noop_samples,
            );
        }

        for &worker_count in &[2usize, 4] {
            // Lifecycle-inclusive comparison: create the pool, execute the same
            // no-op tasks in scheduler-sized batches with a completion barrier,
            // then shut the pool down inside each measured sample. Dedicated worker
            // channels prevent a fast worker from stealing multiple tasks in one
            // batch. Result/error handling and dispatch internals still differ from
            // Scheduler::run_parallel.
            let lifecycle_samples = measure(|| {
                let pool = PersistentPool::new(worker_count);
                pool.run_tasks(task_count, worker_count, noop_task);
                drop(pool);
            });
            report(
                if worker_count == 2 {
                    "persistent_pool_noop_lifecycle_2_workers"
                } else {
                    "persistent_pool_noop_lifecycle_4_workers"
                },
                task_count,
                &lifecycle_samples,
            );

            let pool = PersistentPool::new(worker_count);
            let noop_pool_samples = measure(|| pool.run_tasks(task_count, worker_count, noop_task));
            report(
                if worker_count == 2 {
                    "persistent_pool_noop_2_workers"
                } else {
                    "persistent_pool_noop_4_workers"
                },
                task_count,
                &noop_pool_samples,
            );
            let cpu_pool_samples = measure(|| pool.run_tasks(task_count, worker_count, cpu_task));
            report(
                if worker_count == 2 {
                    "persistent_pool_cpu_1m_2_workers"
                } else {
                    "persistent_pool_cpu_1m_4_workers"
                },
                task_count,
                &cpu_pool_samples,
            );
            drop(pool);
        }

        for &worker_count in &[2usize, 4] {
            let parallel_samples = measure(|| {
                let capacity = NonZeroUsize::new(task_count).expect("task size is nonzero");
                let mut queue = TaskQueue::new(capacity);
                for _ in 0..task_count {
                    queue
                        .try_push(cpu_task as fn() -> Result<u64, &'static str>)
                        .expect("queue capacity matches task count");
                }
                let workers = NonZeroUsize::new(worker_count).expect("worker count is nonzero");
                let results = scheduler.run_parallel(&mut queue, workers);
                black_box(results);
                assert!(queue.is_empty());
            });
            report(
                if worker_count == 2 {
                    "scheduler_queue_parallel_cpu_1m_2_workers"
                } else {
                    "scheduler_queue_parallel_cpu_1m_4_workers"
                },
                task_count,
                &parallel_samples,
            );
        }
    }
}
