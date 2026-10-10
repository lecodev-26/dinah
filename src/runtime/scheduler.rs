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
        let scheduler = Scheduler::new();
        let mut queue = super::super::TaskQueue::new(
            core::num::NonZeroUsize::new(2).expect("positive capacity"),
        );
        let calls = Cell::new(0);
        queue.try_push(|| { calls.set(calls.get() + 1); Ok::<_, ()>(1) }).unwrap();
        queue.try_push(|| Ok::<_, ()>(2)).unwrap();

        assert_eq!(scheduler.dispatch_next(&mut queue), Some(Ok(1)));
        assert_eq!(calls.get(), 1);
        assert_eq!(queue.len(), 1);
        assert_eq!(scheduler.dispatch_next(&mut queue), Some(Ok(2)));
        assert_eq!(scheduler.dispatch_next(&mut queue), None);
    }

    #[test]
    fn dispatch_next_preserves_task_error() {
        let scheduler = Scheduler::new();
        let mut queue = super::super::TaskQueue::new(
            core::num::NonZeroUsize::new(1).expect("positive capacity"),
        );
        queue.try_push(|| Err::<(), _>("queued task failed")).unwrap();
        assert_eq!(scheduler.dispatch_next(&mut queue), Some(Err("queued task failed")));
    }

    #[test]
    fn closed_error_has_stable_display() {
        assert_eq!(ScheduleError::Closed.to_string(), "scheduler is closed");
    }
}
