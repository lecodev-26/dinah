//! Public API contract tests for the Runtime v0.1 surface.
use dinah::runtime::{
    CancellationToken, Executor, RuntimeMetrics, Scheduler, TaskQueue, TaskState,
};
use std::num::NonZeroUsize;

#[test]
fn public_executor_and_scheduler_preserve_task_results_and_errors() {
    let executor = Executor::new();
    assert_eq!(executor.execute(|| Ok::<_, &'static str>(7)), Ok(7));
    assert_eq!(
        executor.execute(|| Err::<(), _>("executor error")),
        Err("executor error")
    );

    let scheduler = Scheduler::new();
    assert_eq!(scheduler.schedule(|| Ok::<_, &'static str>(11)), Ok(Ok(11)));
    assert_eq!(
        scheduler.schedule(|| Err::<(), _>("task error")),
        Ok(Err("task error"))
    );
}

#[test]
fn public_queue_dispatch_and_parallel_execution_preserve_fifo_results() {
    let scheduler = Scheduler::new();
    let mut queue = TaskQueue::new(NonZeroUsize::new(4).expect("positive capacity"));
    for value in 0..4 {
        assert!(queue.try_push(move || Ok::<_, &'static str>(value)).is_ok());
    }

    assert_eq!(
        scheduler.run_parallel(
            &mut queue,
            NonZeroUsize::new(2).expect("positive worker limit")
        ),
        vec![Ok(0), Ok(1), Ok(2), Ok(3)]
    );
    assert!(queue.is_empty());
}

#[test]
fn public_cancellation_state_and_metrics_contracts_are_usable_together() {
    let token = CancellationToken::new();
    let observer = token.clone();
    assert!(token.request_cancellation());
    assert!(observer.is_cancellation_requested());
    assert!(!token.request_cancellation());

    let mut state = TaskState::Pending;
    state
        .transition_to(TaskState::Running)
        .expect("pending task can run");
    state
        .transition_to(TaskState::Cancelled)
        .expect("running task can cancel");
    assert!(state.is_terminal());

    let metrics = RuntimeMetrics::new();
    metrics.record_submitted();
    metrics.record_cancellation_requested();
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.submitted, 1);
    assert_eq!(snapshot.cancellation_requested, 1);
    assert_eq!(snapshot.cancellation_acknowledged, 0);
}
