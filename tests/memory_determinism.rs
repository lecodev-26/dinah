//! Repeatable public Memory contracts for bounded insertion and removal.
use dinah::memory::{Buffer, BufferPushError, SoaBuffer};

#[test]
fn buffer_mutation_sequences_produce_identical_observable_states() {
    const CAPACITY: usize = 32;
    const REPEATS: usize = 64;

    for repetition in 0..REPEATS {
        let mut buffer = Buffer::try_with_capacity(CAPACITY).expect("small reservation succeeds");
        for value in 0..CAPACITY {
            buffer
                .try_push((value * 3 + repetition % 5) as u32)
                .expect("capacity remains available");
        }
        assert_eq!(buffer.try_push(999), Err(BufferPushError::Full(999)));
        let rejected_state = buffer.as_slice().to_vec();
        let popped = buffer.pop();
        buffer.clear();
        buffer
            .try_push(repetition as u32)
            .expect("clear retains logical capacity");
        let observed = (
            rejected_state,
            popped,
            buffer.as_slice().to_vec(),
            buffer.len(),
            buffer.capacity(),
        );

        // Repetition intentionally varies inputs; verify each run against its own
        // deterministic reference rather than assuming different inputs match.
        let expected_popped = Some(((CAPACITY - 1) * 3 + repetition % 5) as u32);
        assert_eq!(observed.1, expected_popped);
        assert_eq!(observed.2, vec![repetition as u32]);
        assert_eq!(observed.3, 1);
        assert_eq!(observed.4, CAPACITY);
        let expected_rejected_state: Vec<u32> = (0..CAPACITY)
            .map(|value| (value * 3 + repetition % 5) as u32)
            .collect();
        assert_eq!(observed.0, expected_rejected_state);
    }
}

#[test]
fn soa_batch_and_pop_keep_column_pairing_repeatably() {
    for repetition in 0..64_u32 {
        let mut rows = SoaBuffer::try_with_capacity(8).expect("small reservation succeeds");
        let first: Vec<u32> = (0..5).map(|i| repetition * 100 + i).collect();
        let second: Vec<u32> = first.iter().map(|value| value ^ 0x55AA).collect();
        rows.try_extend_from_slices(&first, &second)
            .expect("equal batch fits");
        assert_eq!(rows.len(), 5);
        assert_eq!(rows.as_slices(), (first.as_slice(), second.as_slice()));
        assert_eq!(rows.pop(), Some((first[4], second[4])));
        assert_eq!(rows.as_slices(), (&first[..4], &second[..4]));
        rows.clear();
        assert!(rows.is_empty());
        assert_eq!(rows.capacity(), 8);
    }
}
