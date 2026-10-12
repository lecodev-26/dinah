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

#[test]
fn cancellation_task_state_metrics_and_trace_contracts_repeat() {
    use dinah::runtime::{
        CancellationToken, RuntimeMetrics, TaskState, TraceBuffer, TraceEvent, TraceRecordError,
    };

    for _ in 0..64 {
        let token = CancellationToken::new();
        let observer = token.clone();
        assert!(!token.is_cancellation_requested());
        assert!(observer.request_cancellation());
        assert!(token.is_cancellation_requested());
        assert!(!token.request_cancellation());

        let mut state = TaskState::Pending;
        assert!(state.transition_to(TaskState::Running).is_ok());
        assert!(state.transition_to(TaskState::Completed).is_ok());
        assert!(state.is_terminal());
        let invalid = state.transition_to(TaskState::Running).unwrap_err();
        assert_eq!(invalid.from(), TaskState::Completed);
        assert_eq!(invalid.to(), TaskState::Running);
        assert_eq!(state, TaskState::Completed);

        let metrics = RuntimeMetrics::new();
        metrics.record_submitted();
        metrics.record_started();
        metrics.record_completed();
        metrics.record_cancellation_requested();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.submitted, 1);
        assert_eq!(snapshot.started, 1);
        assert_eq!(snapshot.completed, 1);
        assert_eq!(snapshot.failed, 0);
        assert_eq!(snapshot.cancellation_requested, 1);
        assert_eq!(snapshot.cancellation_acknowledged, 0);

        let trace = TraceBuffer::new(NonZeroUsize::new(2).expect("positive trace capacity"));
        trace.record(TraceEvent::TaskSubmitted).unwrap();
        trace.record(TraceEvent::TaskStarted).unwrap();
        assert_eq!(
            trace.record(TraceEvent::TaskFailed),
            Err(TraceRecordError::Full(TraceEvent::TaskFailed))
        );
        assert_eq!(
            trace.snapshot().unwrap(),
            vec![TraceEvent::TaskSubmitted, TraceEvent::TaskStarted]
        );
        assert_eq!(
            trace.drain().unwrap(),
            vec![TraceEvent::TaskSubmitted, TraceEvent::TaskStarted]
        );
        assert!(trace.snapshot().unwrap().is_empty());
        trace.record(TraceEvent::TaskCompleted).unwrap();
        assert_eq!(trace.snapshot().unwrap(), vec![TraceEvent::TaskCompleted]);
    }
}
