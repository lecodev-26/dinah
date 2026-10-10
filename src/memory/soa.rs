use core::fmt;

use super::{Buffer, BufferCreateError};

/// A bounded structure-of-arrays buffer with two contiguous columns.
///
/// Each logical row is stored as a pair, but each field occupies its own
/// contiguous [`Buffer`]. Both columns always have the same length and
/// configured capacity. This is useful when processing one field across many
/// rows; for workloads that usually consume all fields of one row, an array of
/// ordinary structs may be simpler and faster.
///
/// The buffer is not internally synchronized and uses no `unsafe` code.
#[derive(Debug, PartialEq, Eq)]
pub struct SoaBuffer<A, B> {
    first: Buffer<A>,
    second: Buffer<B>,
}

/// Error returned when either column cannot reserve the requested capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoaCreateError {
    /// A column's backing storage could not reserve the requested capacity.
    AllocationFailed,
}

impl fmt::Display for SoaCreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationFailed => f.write_str("failed to reserve SoA column capacity"),
        }
    }
}

impl std::error::Error for SoaCreateError {}

/// A rejected row returned by [`SoaBuffer::try_push`] when the buffer is full.
#[derive(Debug, PartialEq, Eq)]
pub struct SoaPushError<A, B> {
    /// The first column value that was not inserted.
    pub first: A,
    /// The second column value that was not inserted.
    pub second: B,
}

impl<A, B> SoaBuffer<A, B> {
    /// Creates two empty contiguous columns with the same fixed capacity.
    ///
    /// If either reservation fails, no partially constructed buffer is
    /// returned and any reservation already made is released by Rust.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, SoaCreateError> {
        let first = Buffer::try_with_capacity(capacity).map_err(|error| match error {
            BufferCreateError::AllocationFailed => SoaCreateError::AllocationFailed,
        })?;
        let second = Buffer::try_with_capacity(capacity).map_err(|error| match error {
            BufferCreateError::AllocationFailed => SoaCreateError::AllocationFailed,
        })?;
        Ok(Self { first, second })
    }

    /// Appends one logical row to both columns, or returns the untouched row.
    pub fn try_push(&mut self, first: A, second: B) -> Result<(), SoaPushError<A, B>> {
        if self.is_full() {
            return Err(SoaPushError { first, second });
        }
        // Both columns share capacity and length by construction, and only
        // this type can mutate them, so both pushes fit or neither is attempted.
        if self.first.try_push(first).is_err() {
            unreachable!("SoA column invariant violated");
        }
        if self.second.try_push(second).is_err() {
            unreachable!("SoA column invariant violated");
        }
        debug_assert_eq!(self.first.len(), self.second.len());
        Ok(())
    }

    /// Removes and returns the last row, keeping both columns aligned.
    pub fn pop(&mut self) -> Option<(A, B)> {
        let first = self.first.pop()?;
        let second = self.second.pop().expect("SoA column invariant violated");
        Some((first, second))
    }

    /// Removes all rows from both columns while retaining their reservations.
    pub fn clear(&mut self) {
        self.first.clear();
        self.second.clear();
    }

    /// Borrows the initialized values in each contiguous column.
    #[must_use]
    pub fn as_slices(&self) -> (&[A], &[B]) {
        (self.first.as_slice(), self.second.as_slice())
    }

    /// Mutably borrows values in both columns without changing their lengths.
    #[must_use]
    pub fn as_mut_slices(&mut self) -> (&mut [A], &mut [B]) {
        (self.first.as_mut_slice(), self.second.as_mut_slice())
    }

    /// Returns the number of logical rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.first.len()
    }

    /// Returns whether the buffer contains no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.first.is_empty()
    }

    /// Returns the maximum number of logical rows.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.first.capacity()
    }

    /// Returns whether the configured maximum number of rows has been reached.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.first.is_full()
    }
}

#[cfg(test)]
mod tests {
    use super::{SoaBuffer, SoaCreateError, SoaPushError};

    #[test]
    fn keeps_columns_contiguous_and_rows_aligned() {
        let mut rows = SoaBuffer::try_with_capacity(3).unwrap();
        rows.try_push(1_u32, 1.5_f32).unwrap();
        rows.try_push(2, 2.5).unwrap();
        assert_eq!(rows.as_slices(), (&[1, 2][..], &[1.5, 2.5][..]));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows.capacity(), 3);
    }

    #[test]
    fn full_buffer_returns_both_values_without_partial_insertion() {
        let mut rows = SoaBuffer::try_with_capacity(1).unwrap();
        rows.try_push("kept-a", "kept-b").unwrap();
        assert_eq!(
            rows.try_push("rejected-a", "rejected-b"),
            Err(SoaPushError {
                first: "rejected-a",
                second: "rejected-b"
            })
        );
        assert_eq!(rows.as_slices(), (&["kept-a"][..], &["kept-b"][..]));
    }

    #[test]
    fn pop_and_clear_preserve_column_alignment_and_allow_reuse() {
        let mut rows = SoaBuffer::try_with_capacity(2).unwrap();
        rows.try_push(10, "ten").unwrap();
        rows.try_push(20, "twenty").unwrap();
        assert_eq!(rows.pop(), Some((20, "twenty")));
        rows.try_push(30, "thirty").unwrap();
        rows.clear();
        assert!(rows.is_empty());
        rows.try_push(40, "forty").unwrap();
        assert_eq!(rows.as_slices(), (&[40][..], &["forty"][..]));
    }

    #[test]
    fn mutable_columns_update_values_without_changing_shape() {
        let mut rows = SoaBuffer::try_with_capacity(2).unwrap();
        rows.try_push(2, 3).unwrap();
        {
            let (first, second) = rows.as_mut_slices();
            first[0] = 7;
            second[0] = 9;
        }
        assert_eq!(rows.as_slices(), (&[7][..], &[9][..]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows.capacity(), 2);
    }

    #[test]
    fn zero_capacity_is_valid_and_rejects_a_complete_row() {
        let mut rows = SoaBuffer::<u8, u8>::try_with_capacity(0).unwrap();
        assert!(rows.is_full());
        assert_eq!(
            rows.try_push(1, 2),
            Err(SoaPushError {
                first: 1,
                second: 2
            })
        );
        assert_eq!(rows.pop(), None);
    }

    #[test]
    fn impossible_capacity_fails_fallibly() {
        assert_eq!(
            SoaBuffer::<u8, u8>::try_with_capacity(usize::MAX).unwrap_err(),
            SoaCreateError::AllocationFailed
        );
    }
}
