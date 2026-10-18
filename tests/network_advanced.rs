//! Deterministic contracts for reliability, latency, liveness, Runtime and simulated links.
use dinah::network::disconnect::{ConnectionMonitor, DisconnectError, Liveness, ReconnectBackoff};
use dinah::network::latency::LatencyEstimator;
use dinah::network::reliability::{ReliabilityError, ReliableOutbox};
use dinah::network::runtime::NetworkWorkQueue;
use dinah::network::simulator::NetworkSimulator;
use dinah::runtime::Scheduler;
use dinah::simulation::Simulation;
use std::num::NonZeroUsize;
use std::time::Duration;

#[test]
fn reliable_outbox_retains_until_ack_and_retries_only_when_due() {
    let mut outbox = ReliableOutbox::new(3).unwrap();
    assert_eq!(outbox.enqueue("hello", 4), Ok(1));
    assert_eq!(outbox.due_retries(5, 2, 4), Ok(Vec::new()));
    assert_eq!(outbox.due_retries(6, 2, 4), Ok(vec![1]));
    assert_eq!(outbox.due_retries(8, 2, 2), Ok(Vec::new()));
    assert_eq!(outbox.pending().next().unwrap().attempts, 2);
    assert_eq!(outbox.acknowledge(1), 1);
    assert!(outbox.is_empty());
}

#[test]
fn reliable_outbox_rejects_full_and_zero_retry_interval() {
    assert_eq!(
        ReliableOutbox::<u8>::new(0),
        Err(ReliabilityError::ZeroCapacity)
    );
    let mut outbox = ReliableOutbox::new(1).unwrap();
    outbox.enqueue(1, 0).unwrap();
    assert_eq!(outbox.enqueue(2, 0), Err(ReliabilityError::Full));
    assert_eq!(
        outbox.due_retries(1, 0, 3),
        Err(ReliabilityError::ZeroRetryInterval)
    );
}

#[test]
fn latency_estimator_tracks_smoothing_minimum_and_maximum() {
    let mut estimator = LatencyEstimator::new();
    assert_eq!(estimator.smoothed_rtt(), None);
    estimator.record_sample(Duration::from_millis(80));
    estimator.record_sample(Duration::from_millis(160));
    assert_eq!(estimator.sample_count(), 2);
    assert_eq!(estimator.smoothed_rtt(), Some(Duration::from_millis(90)));
    assert_eq!(estimator.minimum_rtt(), Some(Duration::from_millis(80)));
    assert_eq!(estimator.maximum_rtt(), Some(Duration::from_millis(160)));
}

#[test]
fn liveness_timeout_and_activity_are_explicit() {
    let mut monitor =
        ConnectionMonitor::new(Duration::from_secs(5), Duration::from_secs(10)).unwrap();
    assert_eq!(monitor.status(Duration::from_secs(14)), Liveness::Connected);
    assert_eq!(monitor.status(Duration::from_secs(15)), Liveness::TimedOut);
    monitor.observe_activity(Duration::from_secs(16));
    assert_eq!(monitor.status(Duration::from_secs(20)), Liveness::Connected);
    assert_eq!(monitor.status(Duration::from_secs(21)), Liveness::TimedOut);
    assert_eq!(
        ConnectionMonitor::new(Duration::ZERO, Duration::ZERO),
        Err(DisconnectError::ZeroTimeout)
    );
}

#[test]
fn reconnect_backoff_is_exponential_and_capped() {
    let backoff =
        ReconnectBackoff::new(Duration::from_millis(100), Duration::from_millis(350)).unwrap();
    assert_eq!(backoff.delay_for_attempt(0), Duration::from_millis(100));
    assert_eq!(backoff.delay_for_attempt(1), Duration::from_millis(200));
    assert_eq!(backoff.delay_for_attempt(2), Duration::from_millis(350));
    assert_eq!(backoff.delay_for_attempt(20), Duration::from_millis(350));
    assert_eq!(
        ReconnectBackoff::new(Duration::from_millis(200), Duration::from_millis(100)),
        Err(DisconnectError::InvalidBackoffRange)
    );
}

#[test]
fn network_work_queue_dispatches_explicitly_through_runtime() {
    let scheduler = Scheduler::new();
    let mut queue = NetworkWorkQueue::<()>::new(NonZeroUsize::new(2).unwrap());
    assert!(queue.enqueue(Box::new(|| Ok(()))).is_ok());
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.dispatch_next(&scheduler), Some(Ok(Ok(()))));
    assert!(queue.is_empty());
}

#[test]
fn network_work_queue_returns_rejected_work_when_full() {
    let mut queue = NetworkWorkQueue::<()>::new(NonZeroUsize::new(1).unwrap());
    assert!(queue.enqueue(Box::new(|| Ok(()))).is_ok());
    assert!(queue.enqueue(Box::new(|| Ok(()))).is_err());
    assert_eq!(queue.len(), 1);
}

#[test]
fn simulator_applies_scripted_loss_and_tick_latency_deterministically() {
    let mut link = NetworkSimulator::new();
    assert_eq!(link.send(2, 3, "delayed", false), Ok(Some(0)));
    assert_eq!(link.send(2, 0, "dropped", true), Ok(None));
    assert_eq!(link.receive_due(4), Vec::<&str>::new());
    assert_eq!(link.receive_due(5), vec!["delayed"]);
    assert_eq!(link.dropped_count(), 1);
    assert_eq!(link.delivered_count(), 1);
}

#[test]
fn simulator_orders_same_tick_delivery_by_send_order() {
    let mut link = NetworkSimulator::new();
    link.send(1, 2, 10, false).unwrap();
    link.send(1, 1, 20, false).unwrap();
    link.send(2, 0, 30, false).unwrap();
    assert_eq!(link.receive_due(2), vec![20, 30]);
    assert_eq!(link.receive_due(3), vec![10]);
}

#[test]
fn simulator_integrates_by_explicit_simulation_tick() {
    let mut simulation = Simulation::new((), Duration::from_millis(16)).unwrap();
    let mut link = NetworkSimulator::new();
    link.send(0, 1, "input", false).unwrap();
    assert!(link.receive_due_for_simulation(&simulation).is_empty());
    simulation.step(|_, _| Ok::<(), ()>(())).unwrap();
    assert_eq!(link.receive_due_for_simulation(&simulation), vec!["input"]);
}
