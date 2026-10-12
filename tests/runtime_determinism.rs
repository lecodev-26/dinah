//! Repeatable ordering contracts for public Runtime parallel dispatch.
use dinah::runtime::{Scheduler, TaskQueue};
use std::num::NonZeroUsize;

#[test]
fn parallel_dispatch_returns_fifo_results_despite_different_task_workloads() {
    const TASKS: usize = 64;
    let expected: Vec<Result<usize, usize>> = (0..TASKS)
        .map(|index| {
            if index % 11 == 0 {
                Err(index)
            } else {
                Ok(index)
            }
        })
        .collect();

    for worker_count in [1, 2, 3, 7, 64] {
        for repetition in 0..16 {
            let scheduler = Scheduler::new();
            let mut queue =
                TaskQueue::new(NonZeroUsize::new(TASKS).expect("task queue capacity is positive"));

            for index in 0..TASKS {
                let workload = ((TASKS - index + repetition) % 17) * 200;
                queue
                    .try_push(move || {
                        // Vary work per task without depending on wall-clock timing.
                        let mut accumulator = index;
                        for step in 0..workload {
                            accumulator = accumulator.wrapping_add(step ^ index);
                        }
                        std::hint::black_box(accumulator);
                        if index % 11 == 0 {
                            Err(index)
                        } else {
                            Ok(index)
                        }
                    })
                    .unwrap_or_else(|_| panic!("queue capacity matches task count"));
            }

            let results = scheduler.run_parallel(
                &mut queue,
                NonZeroUsize::new(worker_count).expect("worker count is positive"),
            );
            assert_eq!(
                results, expected,
                "workers={worker_count}, repetition={repetition}"
            );
            assert!(queue.is_empty(), "workers={worker_count}");
        }
    }
}
