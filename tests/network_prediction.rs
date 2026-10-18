//! Deterministic contract tests for bounded client prediction/reconciliation.
use dinah::network::prediction::{
    AuthoritativeSnapshot, PredictionBuffer, PredictionError, ReconcileOutcome,
};

fn add(state: &mut i64, input: &i64, _tick: u64) {
    *state += input;
}

#[test]
fn predicts_inputs_immediately_with_monotonic_sequences() {
    let mut prediction = PredictionBuffer::new(0_i64, 4).expect("valid capacity");
    assert_eq!(prediction.predict(1, 3, add), Ok(1));
    assert_eq!(prediction.predict(2, 5, add), Ok(2));
    assert_eq!(*prediction.state(), 8);
    assert_eq!(prediction.last_issued_sequence(), 2);
    assert_eq!(prediction.pending_len(), 2);
}

#[test]
fn reconciliation_drops_acknowledged_inputs_and_replays_pending_inputs() {
    let mut prediction = PredictionBuffer::new(0_i64, 8).expect("valid capacity");
    prediction.predict(1, 2, add).expect("predict input 1");
    prediction.predict(2, 4, add).expect("predict input 2");
    prediction.predict(3, 8, add).expect("predict input 3");

    let result = prediction
        .reconcile(
            AuthoritativeSnapshot {
                tick: 2,
                last_processed_input: 2,
                state: 6,
            },
            add,
        )
        .expect("reconcile authoritative snapshot");

    assert_eq!(result, ReconcileOutcome::Applied { replayed_inputs: 1 });
    assert_eq!(*prediction.state(), 14);
    assert_eq!(prediction.pending_len(), 1);
    assert_eq!(prediction.pending_inputs().next().unwrap().sequence, 3);
    assert_eq!(prediction.last_acknowledged_sequence(), 2);
}

#[test]
fn stale_snapshot_does_not_rewind_state_or_history() {
    let mut prediction = PredictionBuffer::new(0_i64, 4).expect("valid capacity");
    prediction.predict(1, 7, add).expect("predict input");
    prediction
        .reconcile(
            AuthoritativeSnapshot {
                tick: 10,
                last_processed_input: 1,
                state: 7,
            },
            add,
        )
        .expect("accept first snapshot");
    let outcome = prediction
        .reconcile(
            AuthoritativeSnapshot {
                tick: 9,
                last_processed_input: 0,
                state: 100,
            },
            add,
        )
        .expect("ignore stale snapshot");
    assert_eq!(outcome, ReconcileOutcome::StaleSnapshot);
    assert_eq!(*prediction.state(), 7);
    assert_eq!(prediction.pending_len(), 0);
}

#[test]
fn history_capacity_rejects_new_input_without_changing_state() {
    let mut prediction = PredictionBuffer::new(10_i64, 1).expect("valid capacity");
    prediction.predict(1, 2, add).expect("first input");
    assert_eq!(
        prediction.predict(2, 5, add),
        Err(PredictionError::HistoryFull)
    );
    assert_eq!(*prediction.state(), 12);
    assert_eq!(prediction.last_issued_sequence(), 1);
    assert_eq!(prediction.pending_len(), 1);
}

#[test]
fn zero_capacity_and_future_acknowledgement_are_rejected() {
    assert!(matches!(
        PredictionBuffer::<i64, i64>::new(0, 0),
        Err(PredictionError::ZeroCapacity)
    ));
    let mut prediction = PredictionBuffer::new(0_i64, 2).expect("valid capacity");
    assert_eq!(
        prediction.reconcile(
            AuthoritativeSnapshot {
                tick: 1,
                last_processed_input: 1,
                state: 99,
            },
            add,
        ),
        Err(PredictionError::InvalidAcknowledgement)
    );
    assert_eq!(*prediction.state(), 0);
    assert_eq!(prediction.pending_len(), 0);
}

#[test]
fn same_initial_state_and_inputs_replay_to_same_result() {
    let mut first = PredictionBuffer::new(10_i64, 4).expect("valid capacity");
    let mut second = PredictionBuffer::new(10_i64, 4).expect("valid capacity");
    for (tick, input) in [(1, 2), (2, -1), (3, 8)] {
        first.predict(tick, input, add).expect("first prediction");
        second.predict(tick, input, add).expect("second prediction");
    }
    assert_eq!(first.state(), second.state());
    first
        .reconcile(
            AuthoritativeSnapshot {
                tick: 1,
                last_processed_input: 1,
                state: 12,
            },
            add,
        )
        .expect("replay first history");
    second
        .reconcile(
            AuthoritativeSnapshot {
                tick: 1,
                last_processed_input: 1,
                state: 12,
            },
            add,
        )
        .expect("replay second history");
    assert_eq!(first.state(), second.state());
    assert_eq!(*first.state(), 19);
}
