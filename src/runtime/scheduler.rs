use super::{Executor, TaskQueue};

/// A small synchronous scheduler that dispatches each submitted task immediately.
///
/// This initial scheduler owns no queue and creates no threads. Submission
/// executes on the caller's thread, so it provides deterministic, sequential
/// dispatch without claiming FIFO ordering across concurrent callers, fairness,
/// cancellation, or background progress.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Scheduler {
    executor: Executor,
}

/// Error returned when a task cannot be submitted to the scheduler.
///
/// The initial immediate scheduler has no capacity limit, so this type is
/// reserved for explicit scheduler-level rejection as queueing is introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleError {
    /// The scheduler is not accepting work. The initial scheduler never emits
    /// this error; it is reserved for a future lifecycle-aware implementation.
    Closed,
}

impl core::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Closed => f.write_str("scheduler is closed"),
        }
    }
}

impl std::error::Error for ScheduleError {}

impl Scheduler {
    /// Creates a scheduler that immediately executes submitted tasks.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            executor: Executor::new(),
        }
    }

    /// Submits a one-shot task for immediate synchronous execution.
    ///
    /// The returned result is the task's own result; scheduler rejection is
    /// represented by [`ScheduleError`] separately in future queued variants.
    /// In this first implementation, submission always dispatches immediately
    /// and the scheduler-level error type is not produced.
    pub fn schedule<T, E, F>(&self, task: F) -> Result<Result<T, E>, ScheduleError>
    where
        F: FnOnce() -> Result<T, E>,
    {
        Ok(self.executor.execute(task))
    }

    /// Removes and executes the oldest queued task, if one is available.
    ///
    /// The queue item is removed before user code runs. The queue is borrowed
    /// only for the pop operation, so task execution does not occur while any
    /// queue borrow or internal queue lock is held. `None` means the queue was
    /// empty; `Some(Err(error))` is the task's own error.
    pub fn dispatch_next<T, E, F>(&self, queue: &mut TaskQueue<F>) -> Option<Result<T, E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        queue.pop().map(|task| self.executor.execute(task))
    }

    /// Executes queued tasks sequentially until the queue is empty.
    ///
    /// Results are returned in dispatch order, including task-level errors;
    /// an error does not stop later queued tasks from running. The queue is
    /// drained one item at a time, and user code runs after each item has been
    /// removed from the queue. This method does not add concurrency or a worker
    /// pool.
    pub fn run_until_empty<T, E, F>(&self, queue: &mut TaskQueue<F>) -> Vec<Result<T, E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let mut results = Vec::with_capacity(queue.len());
        while let Some(result) = self.dispatch_next(queue) {
            results.push(result);
        }
        results
    }

    /// Executes queued tasks in parallel batches bounded by `max_workers`.
    ///
    /// A bounded set of scoped workers is created once per call and reused across
    /// batches. Each batch is removed from the queue before its tasks start, and
    /// results are returned in FIFO dispatch order regardless of completion
    /// order. Task-level `Err` is retained and does not stop later batches. If a
    /// task panics, all tasks in its current batch are allowed to finish, workers
    /// are shut down, and the panic is propagated; later batches remain queued.
    /// Workers are scoped to this call, not retained across calls, so closures may
    /// borrow data and no persistent scheduler lifecycle is implied.
    pub fn run_parallel<T, E, F>(
        &self,
        queue: &mut TaskQueue<F>,
        max_workers: core::num::NonZeroUsize,
    ) -> Vec<Result<T, E>>
    where
        T: Send,
        E: Send,
        F: FnOnce() -> Result<T, E> + Send,
    {
        use std::any::Any;
        use std::panic::{catch_unwind, AssertUnwindSafe};
        use std::sync::{mpsc, Arc, Mutex};

        if queue.is_empty() {
            return Vec::new();
        }

        let total_tasks = queue.len();
        let worker_count = max_workers.get().min(total_tasks);
        let mut results: Vec<Option<Result<T, E>>> =
            std::iter::repeat_with(|| None).take(total_tasks).collect();

        std::thread::scope(|scope| {
            let (task_sender, task_receiver) = mpsc::channel::<Option<(usize, F)>>();
            let task_receiver = Arc::new(Mutex::new(task_receiver));
            let (result_sender, result_receiver) =
                mpsc::channel::<(usize, Result<Result<T, E>, Box<dyn Any + Send>>)>();

            for _ in 0..worker_count {
                let task_receiver = Arc::clone(&task_receiver);
                let result_sender = result_sender.clone();
                scope.spawn(move || loop {
                    let message = task_receiver
                        .lock()
                        .expect("scoped worker receiver mutex is not poisoned")
                        .recv();
                    match message {
                        Ok(Some((index, task))) => {
                            let outcome = catch_unwind(AssertUnwindSafe(task));
                            if result_sender.send((index, outcome)).is_err() {
                                break;
                            }
                        }
                        Ok(None) | Err(_) => break,
                    }
                });
            }
            drop(result_sender);

            let mut next_index = 0;
            // Reuse the batch allocation across iterations. Each batch must be
            // fully removed from the queue before any task is sent to a worker.
            let mut batch = Vec::with_capacity(worker_count);
            while !queue.is_empty() {
                let batch_size = queue.len().min(worker_count);
                batch.clear();
                for _ in 0..batch_size {
                    if let Some(task) = queue.pop() {
                        batch.push(task);
                    }
                }

                for task in batch.drain(..) {
                    task_sender
                        .send(Some((next_index, task)))
                        .expect("scoped workers accept the current batch");
                    next_index += 1;
                }

                let mut panics = Vec::new();
                for _ in 0..batch_size {
                    let (index, outcome) = result_receiver
                        .recv()
                        .expect("every dispatched task reports an outcome");
                    match outcome {
                        Ok(result) => results[index] = Some(result),
                        Err(payload) => panics.push((index, payload)),
                    }
                }

                if !panics.is_empty() {
                    for _ in 0..worker_count {
                        let _ = task_sender.send(None);
                    }
                    panics.sort_unstable_by_key(|(index, _)| *index);
                    std::panic::resume_unwind(panics.remove(0).1);
                }
            }

            for _ in 0..worker_count {
                task_sender
                    .send(None)
                    .expect("scoped workers accept shutdown messages");
            }
        });

        results
            .into_iter()
            .map(|result| result.expect("every normally completed task has a result"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{ScheduleError, Scheduler};
    use core::cell::Cell;

    #[test]
    fn schedules_task_immediately_and_returns_result() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.schedule(|| Ok::<_, &'static str>(7)), Ok(Ok(7)));
    }

    #[test]
    fn executes_submitted_task_once() {
        let scheduler = Scheduler::new();
        let calls = Cell::new(0);

        let result = scheduler.schedule(|| {
            calls.set(calls.get() + 1);
            Ok::<_, ()>(())
        });

        assert_eq!(result, Ok(Ok(())));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn preserves_task_error_separately_from_scheduler_error() {
        let scheduler = Scheduler::new();
        assert_eq!(
            scheduler.schedule(|| Err::<(), _>("task failed")),
            Ok(Err("task failed"))
        );
    }

    #[test]
    fn dispatch_next_returns_none_for_empty_queue() {
        let scheduler = Scheduler::new();
        let mut queue = super::super::TaskQueue::<fn() -> Result<(), ()>>::new(
            core::num::NonZeroUsize::new(1).expect("positive capacity"),
        );
        assert_eq!(scheduler.dispatch_next(&mut queue), None);
    }

    #[test]
    fn dispatch_next_executes_oldest_task_once() {
        fn first() -> Result<i32, ()> {
            Ok(1)
        }
        fn second() -> Result<i32, ()> {
            Ok(2)
        }

        let scheduler = Scheduler::new();
        let mut queue: super::super::TaskQueue<fn() -> Result<i32, ()>> =
            super::super::TaskQueue::new(
                core::num::NonZeroUsize::new(2).expect("positive capacity"),
            );
        assert!(queue.try_push(first).is_ok());
        assert!(queue.try_push(second).is_ok());

        assert_eq!(scheduler.dispatch_next(&mut queue), Some(Ok(1)));
        assert_eq!(queue.len(), 1);
        assert_eq!(scheduler.dispatch_next(&mut queue), Some(Ok(2)));
        assert_eq!(scheduler.dispatch_next(&mut queue), None);
    }

    #[test]
    fn run_until_empty_drains_in_fifo_order_and_preserves_errors() {
        fn first() -> Result<i32, &'static str> {
            Ok(1)
        }
        fn failed() -> Result<i32, &'static str> {
            Err("failed")
        }
        fn third() -> Result<i32, &'static str> {
            Ok(3)
        }

        let scheduler = Scheduler::new();
        let mut queue: super::super::TaskQueue<fn() -> Result<i32, &'static str>> =
            super::super::TaskQueue::new(
                core::num::NonZeroUsize::new(3).expect("positive capacity"),
            );
        assert!(queue.try_push(first).is_ok());
        assert!(queue.try_push(failed).is_ok());
        assert!(queue.try_push(third).is_ok());

        assert_eq!(
            scheduler.run_until_empty(&mut queue),
            vec![Ok(1), Err("failed"), Ok(3)]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn run_until_empty_returns_no_results_for_empty_queue() {
        let scheduler = Scheduler::new();
        let mut queue = super::super::TaskQueue::<fn() -> Result<(), ()>>::new(
            core::num::NonZeroUsize::new(1).expect("positive capacity"),
        );
        assert!(scheduler.run_until_empty(&mut queue).is_empty());
    }

    #[test]
    fn run_parallel_preserves_fifo_result_order_and_task_errors() {
        fn first() -> Result<i32, &'static str> {
            Ok(1)
        }
        fn failed() -> Result<i32, &'static str> {
            Err("failed")
        }
        fn third() -> Result<i32, &'static str> {
            Ok(3)
        }

        let scheduler = Scheduler::new();
        let mut queue: super::super::TaskQueue<fn() -> Result<i32, &'static str>> =
            super::super::TaskQueue::new(
                core::num::NonZeroUsize::new(3).expect("positive capacity"),
            );
        assert!(queue.try_push(first).is_ok());
        assert!(queue.try_push(failed).is_ok());
        assert!(queue.try_push(third).is_ok());

        assert_eq!(
            scheduler.run_parallel(
                &mut queue,
                core::num::NonZeroUsize::new(2).expect("positive worker limit"),
            ),
            vec![Ok(1), Err("failed"), Ok(3)]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn run_parallel_executes_tasks_concurrently_within_worker_limit() {
        use std::sync::{Arc, Barrier};
        let scheduler = Scheduler::new();
        let barrier = Arc::new(Barrier::new(2));
        let mut queue = super::super::TaskQueue::new(core::num::NonZeroUsize::new(4).unwrap());
        for value in 0..4 {
            let barrier = Arc::clone(&barrier);
            assert!(queue
                .try_push(move || {
                    barrier.wait();
                    Ok::<_, &'static str>(value)
                })
                .is_ok());
        }
        let results = scheduler.run_parallel(&mut queue, core::num::NonZeroUsize::new(2).unwrap());
        assert_eq!(results, vec![Ok(0), Ok(1), Ok(2), Ok(3)]);
        assert!(queue.is_empty());
    }

    #[test]
    fn run_parallel_continues_after_error_across_batch_boundaries() {
        fn first() -> Result<i32, &'static str> {
            Ok(10)
        }
        fn failed() -> Result<i32, &'static str> {
            Err("batch one failed")
        }
        fn third() -> Result<i32, &'static str> {
            Ok(30)
        }
        fn fourth() -> Result<i32, &'static str> {
            Ok(40)
        }

        let scheduler = Scheduler::new();
        let mut queue: super::super::TaskQueue<fn() -> Result<i32, &'static str>> =
            super::super::TaskQueue::new(
                core::num::NonZeroUsize::new(4).expect("positive capacity"),
            );
        for task in [first, failed, third, fourth] {
            assert!(queue.try_push(task).is_ok());
        }

        let results = scheduler.run_parallel(
            &mut queue,
            core::num::NonZeroUsize::new(2).expect("positive worker limit"),
        );
        assert_eq!(
            results,
            vec![Ok(10), Err("batch one failed"), Ok(30), Ok(40)]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn run_parallel_with_one_worker_preserves_fifo_across_batches() {
        fn task(value: i32) -> impl FnOnce() -> Result<i32, ()> {
            move || Ok(value)
        }

        let scheduler = Scheduler::new();
        let mut queue = super::super::TaskQueue::new(
            core::num::NonZeroUsize::new(3).expect("positive capacity"),
        );
        for value in 0..3 {
            assert!(queue.try_push(task(value)).is_ok());
        }

        assert_eq!(
            scheduler.run_parallel(
                &mut queue,
                core::num::NonZeroUsize::new(1).expect("positive worker limit"),
            ),
            vec![Ok(0), Ok(1), Ok(2)]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn run_parallel_reuses_workers_and_propagates_panic_after_current_batch() {
        use std::panic::{catch_unwind, AssertUnwindSafe};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let scheduler = Scheduler::new();
        let completed = Arc::new(AtomicUsize::new(0));
        let mut queue: super::super::TaskQueue<Box<dyn FnOnce() -> Result<(), ()> + Send>> =
            super::super::TaskQueue::new(
                core::num::NonZeroUsize::new(3).expect("positive capacity"),
            );

        assert!(queue
            .try_push(Box::new(|| -> Result<(), ()> {
                panic!("expected task panic")
            }))
            .is_ok());
        let completed_in_batch = Arc::clone(&completed);
        assert!(queue
            .try_push(Box::new(move || {
                completed_in_batch.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(())
            }))
            .is_ok());
        let queued_for_later = Arc::clone(&completed);
        assert!(queue
            .try_push(Box::new(move || {
                queued_for_later.fetch_add(1, Ordering::SeqCst);
                Ok::<_, ()>(())
            }))
            .is_ok());

        let outcome = catch_unwind(AssertUnwindSafe(|| {
            scheduler.run_parallel(
                &mut queue,
                core::num::NonZeroUsize::new(2).expect("positive worker limit"),
            )
        }));

        assert!(outcome.is_err());
        assert_eq!(completed.load(Ordering::SeqCst), 1);
        assert_eq!(queue.len(), 1, "later batches remain queued after a panic");
        assert_eq!(scheduler.run_until_empty(&mut queue), vec![Ok(())]);
        assert_eq!(completed.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn run_parallel_handles_empty_queue() {
        let scheduler = Scheduler::new();
        let mut queue = super::super::TaskQueue::<fn() -> Result<(), ()>>::new(
            core::num::NonZeroUsize::new(1).expect("positive capacity"),
        );
        assert!(scheduler
            .run_parallel(
                &mut queue,
                core::num::NonZeroUsize::new(2).expect("positive worker limit"),
            )
            .is_empty());
    }

    #[test]
    fn dispatch_next_preserves_task_error() {
        fn failed() -> Result<(), &'static str> {
            Err("queued task failed")
        }

        let scheduler = Scheduler::new();
        let mut queue: super::super::TaskQueue<fn() -> Result<(), &'static str>> =
            super::super::TaskQueue::new(
                core::num::NonZeroUsize::new(1).expect("positive capacity"),
            );
        assert!(queue.try_push(failed).is_ok());
        assert_eq!(
            scheduler.dispatch_next(&mut queue),
            Some(Err("queued task failed"))
        );
    }

    #[test]
    fn closed_error_has_stable_display() {
        assert_eq!(ScheduleError::Closed.to_string(), "scheduler is closed");
    }
}
