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
    /// Each batch is removed from the queue before its tasks start. Tasks in a
    /// batch run on scoped threads, and results are collected in FIFO dispatch
    /// order regardless of completion order. A task-level `Err` is retained and
    /// does not stop later batches. Task closures and their result types must be
    /// `Send`; panics are not converted into task errors and propagate to the
    /// caller. This creates temporary threads per batch, not a persistent pool.
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
        let mut results = Vec::with_capacity(queue.len());
        while !queue.is_empty() {
            let batch_size = queue.len().min(max_workers.get());
            let mut batch = Vec::with_capacity(batch_size);
            for _ in 0..batch_size {
                if let Some(task) = queue.pop() {
                    batch.push(task);
                }
            }

            let mut batch_results = std::thread::scope(|scope| {
                let handles: Vec<_> = batch
                    .into_iter()
                    .map(|task| scope.spawn(move || task()))
                    .collect();
                handles
                    .into_iter()
                    .map(|handle| match handle.join() {
                        Ok(result) => result,
                        Err(payload) => std::panic::resume_unwind(payload),
                    })
                    .collect::<Vec<_>>()
            });
            results.append(&mut batch_results);
        }
        results
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
