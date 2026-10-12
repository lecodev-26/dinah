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

#[test]
fn buffer_bulk_extend_errors_are_repeatable_and_leave_state_unchanged() {
    use dinah::memory::BufferExtendError;

    for repetition in 0..64_u32 {
        let mut buffer = Buffer::try_with_capacity(3).expect("small reservation succeeds");
        buffer.try_push(repetition).expect("one slot is available");
        buffer
            .try_extend_from_slice(&[repetition + 1])
            .expect("second element fits");
        let before = buffer.as_slice().to_vec();
        assert_eq!(
            buffer.try_extend_from_slice(&[repetition + 2, repetition + 3]),
            Err(BufferExtendError {
                requested: 2,
                remaining: 1,
            })
        );
        assert_eq!(buffer.as_slice(), before.as_slice());
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.capacity(), 3);
    }
}

#[test]
fn soa_batch_validation_errors_never_desynchronize_columns() {
    use dinah::memory::SoaExtendError;

    for repetition in 0..64_u32 {
        let mut rows = SoaBuffer::try_with_capacity(3).expect("small reservation succeeds");
        rows.try_push(repetition, repetition ^ 0xFFFF)
            .expect("first row fits");
        let before = {
            let (first, second) = rows.as_slices();
            (first.to_vec(), second.to_vec())
        };

        assert_eq!(
            rows.try_extend_from_slices(&[repetition + 1, repetition + 2], &[9]),
            Err(SoaExtendError::LengthMismatch {
                first: 2,
                second: 1,
            })
        );
        assert_eq!(
            rows.try_extend_from_slices(
                &[repetition + 1, repetition + 2, repetition + 3],
                &[1, 2, 3]
            ),
            Err(SoaExtendError::CapacityExceeded {
                requested: 3,
                remaining: 2,
            })
        );
        let (first, second) = rows.as_slices();
        assert_eq!((first, second), (before.0.as_slice(), before.1.as_slice()));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows.capacity(), 3);
    }
}

#[test]
fn zero_capacity_buffers_have_repeatable_empty_and_full_contracts() {
    for _ in 0..64 {
        let mut buffer = Buffer::<u8>::try_with_capacity(0).expect("zero capacity is valid");
        assert!(buffer.is_empty());
        assert!(buffer.is_full());
        assert_eq!(buffer.capacity(), 0);
        assert_eq!(buffer.try_push(7), Err(BufferPushError::Full(7)));
        assert_eq!(buffer.try_extend_from_slice(&[]), Ok(()));
        assert!(buffer.as_slice().is_empty());

        let mut rows = SoaBuffer::<u8, u8>::try_with_capacity(0).expect("zero capacity is valid");
        assert!(rows.is_empty());
        assert!(rows.is_full());
        assert_eq!(rows.capacity(), 0);
        assert_eq!(rows.try_push(7, 8).unwrap_err().first, 7);
        assert_eq!(rows.as_slices(), (&[][..], &[][..]));
        assert_eq!(rows.try_extend_from_slices(&[], &[]), Ok(()));
    }
}
