use core::fmt;

/// A fixed-logical-capacity, contiguous buffer that owns its elements.
///
/// The backing `Vec` reserves the configured maximum at construction, so
/// successful construction makes pushes up to that limit avoid growth-related
/// reallocations. The allocator may reserve more than requested. Capacity is
/// an element limit, not a byte budget or a residency guarantee.
///
/// This type is not internally synchronized. Mutations require exclusive
/// access; [`Self::as_slice`] and [`Self::as_mut_slice`] lend access only to
/// initialized elements without exposing the backing allocation or its capacity.
///
/// The buffer intentionally does not implement [`Clone`]: an implicit clone
/// could duplicate storage without fallibly reserving the configured maximum,
/// breaking the no-growth guarantee for subsequent pushes. Move the buffer to
/// transfer ownership; use an explicit element-level copy when duplication is
/// required.
///
/// A slice borrow cannot outlive the buffer or overlap a mutable borrow:
///
/// ```compile_fail
/// use dinah::memory::Buffer;
///
/// let mut buffer = Buffer::try_with_capacity(2).unwrap();
/// buffer.try_push(10).unwrap();
/// let view = buffer.as_slice();
/// buffer.try_push(20).unwrap(); // cannot mutably borrow while `view` is live
/// println!("{view:?}");
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Buffer<T> {
    items: Vec<T>,
    max_capacity: usize,
}

/// Error returned when the requested buffer capacity cannot be reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferCreateError {
    /// The allocator could not satisfy the requested reservation.
    AllocationFailed,
}

impl fmt::Display for BufferCreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationFailed => f.write_str("failed to reserve buffer capacity"),
        }
    }
}

impl std::error::Error for BufferCreateError {}

/// Failure to append an item to a [`Buffer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferPushError<T> {
    /// The configured element capacity has been reached; returns the item.
    Full(T),
}

/// Failure to append a slice to a [`Buffer`] without exceeding its capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferExtendError {
    /// Number of elements requested by the failed operation.
    pub requested: usize,
    /// Number of additional elements the buffer can currently hold.
    pub remaining: usize,
}

impl fmt::Display for BufferExtendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot append {} elements; only {} slots remain",
            self.requested, self.remaining
        )
    }
}

impl std::error::Error for BufferExtendError {}

impl<T> Buffer<T> {
    /// Creates an empty buffer with a fixed maximum element capacity.
    ///
    /// Capacity zero is valid. Reservation failure is reported rather than
    /// handled by an infallible allocation path.
    pub fn try_with_capacity(max_capacity: usize) -> Result<Self, BufferCreateError> {
        let mut items = Vec::new();
        items
            .try_reserve_exact(max_capacity)
            .map_err(|_| BufferCreateError::AllocationFailed)?;
        Ok(Self {
            items,
            max_capacity,
        })
    }

    /// Appends an item unless the configured maximum has been reached.
    /// Returns the rejected item unchanged when full.
    pub fn try_push(&mut self, item: T) -> Result<(), BufferPushError<T>> {
        if self.is_full() {
            return Err(BufferPushError::Full(item));
        }
        self.items.push(item);
        Ok(())
    }

    /// Appends a slice in one operation if every element fits.
    ///
    /// If the slice exceeds the remaining logical capacity, the buffer is left
    /// unchanged and the error reports the requested and remaining element
    /// counts. The element clone operation may panic; as with standard Rust
    /// collection operations, this method does not promise rollback on panic.
    /// The configured reservation means successful appends within the logical
    /// capacity do not require buffer growth.
    pub fn try_extend_from_slice(&mut self, values: &[T]) -> Result<(), BufferExtendError>
    where
        T: Clone,
    {
        let remaining = self.capacity() - self.len();
        if values.len() > remaining {
            return Err(BufferExtendError {
                requested: values.len(),
                remaining,
            });
        }
        self.items.extend_from_slice(values);
        Ok(())
    }

    /// Removes and returns the last item, if present.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Removes all items while retaining the reserved backing storage.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Returns the initialized elements as a contiguous read-only slice.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self.items.as_slice()
    }

    /// Returns the initialized elements as a contiguous mutable slice.
    ///
    /// This allows in-place element updates but cannot change the buffer's
    /// length, backing allocation, or configured capacity.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.items.as_mut_slice()
    }

    /// Returns the number of initialized elements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the buffer contains no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the configured maximum number of elements.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Returns whether the configured maximum has been reached.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, BufferCreateError, BufferExtendError, BufferPushError};

    #[test]
    fn starts_empty_with_configured_capacity() {
        let buffer = Buffer::<u8>::try_with_capacity(3).unwrap();
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 3);
        assert_eq!(buffer.as_slice(), &[]);
    }

    #[test]
    fn preserves_contiguous_insertion_order() {
        let mut buffer = Buffer::try_with_capacity(3).unwrap();
        buffer.try_push(11).unwrap();
        buffer.try_push(22).unwrap();
        buffer.try_push(33).unwrap();
        assert_eq!(buffer.as_slice(), &[11, 22, 33]);
        assert!(buffer.is_full());
    }

    #[test]
    fn extends_contiguously_in_one_bounded_operation() {
        let mut buffer = Buffer::try_with_capacity(5).unwrap();
        buffer.try_push(1).unwrap();
        buffer.try_extend_from_slice(&[2, 3, 4]).unwrap();
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4]);
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.capacity(), 5);
    }

    #[test]
    fn failed_bulk_extend_leaves_contents_unchanged() {
        let mut buffer = Buffer::try_with_capacity(2).unwrap();
        buffer.try_push(10).unwrap();
        let error = buffer.try_extend_from_slice(&[20, 30]).unwrap_err();
        assert_eq!(
            error,
            BufferExtendError {
                requested: 2,
                remaining: 1
            }
        );
        assert_eq!(buffer.as_slice(), &[10]);
    }

    #[test]
    fn mutable_slice_changes_values_without_changing_shape() {
        let mut buffer = Buffer::try_with_capacity(3).unwrap();
        buffer.try_extend_from_slice(&[2, 4]).unwrap();
        buffer.as_mut_slice()[0] = 7;
        assert_eq!(buffer.as_slice(), &[7, 4]);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.capacity(), 3);
    }

    #[test]
    fn empty_bulk_extend_is_valid_at_zero_capacity() {
        let mut buffer = Buffer::<u8>::try_with_capacity(0).unwrap();
        assert_eq!(buffer.try_extend_from_slice(&[]), Ok(()));
        assert_eq!(buffer.as_slice(), &[]);
    }

    #[test]
    fn full_buffer_returns_item_without_changing_contents() {
        let mut buffer = Buffer::try_with_capacity(1).unwrap();
        buffer.try_push("kept").unwrap();
        assert_eq!(
            buffer.try_push("returned"),
            Err(BufferPushError::Full("returned"))
        );
        assert_eq!(buffer.as_slice(), &["kept"]);
    }

    #[test]
    fn capacity_can_be_reused_after_pop_and_clear() {
        let mut buffer = Buffer::try_with_capacity(2).unwrap();
        buffer.try_push(1).unwrap();
        buffer.try_push(2).unwrap();
        assert_eq!(buffer.pop(), Some(2));
        buffer.try_push(3).unwrap();
        buffer.clear();
        assert!(buffer.is_empty());
        buffer.try_push(4).unwrap();
        assert_eq!(buffer.as_slice(), &[4]);
    }

    #[test]
    fn zero_capacity_is_valid_but_rejects_every_item() {
        let mut buffer = Buffer::<u8>::try_with_capacity(0).unwrap();
        assert!(buffer.is_full());
        assert_eq!(buffer.try_push(7), Err(BufferPushError::Full(7)));
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn impossible_capacity_fails_fallibly() {
        assert_eq!(
            Buffer::<u8>::try_with_capacity(usize::MAX).unwrap_err(),
            BufferCreateError::AllocationFailed
        );
    }

    #[test]
    fn zero_sized_types_obey_logical_capacity() {
        let mut buffer = Buffer::<()>::try_with_capacity(3).unwrap();
        assert_eq!(buffer.capacity(), 3);
        for _ in 0..3 {
            buffer.try_push(()).unwrap();
        }
        assert_eq!(buffer.len(), 3);
        assert!(buffer.is_full());
        assert_eq!(buffer.try_push(()), Err(BufferPushError::Full(())));
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.pop(), Some(()));
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn over_aligned_elements_keep_their_type_alignment() {
        #[repr(align(128))]
        #[derive(Debug)]
        struct Aligned(u64);

        let mut buffer = Buffer::try_with_capacity(2).unwrap();
        buffer.try_push(Aligned(17)).unwrap();
        buffer.try_push(Aligned(23)).unwrap();
        let values = buffer.as_slice();
        assert_eq!(
            (values.as_ptr() as usize) % core::mem::align_of::<Aligned>(),
            0
        );
        assert_eq!(
            (core::ptr::addr_of!(values[1]) as usize) % core::mem::align_of::<Aligned>(),
            0
        );
        assert_eq!(
            values.iter().map(|item| item.0).collect::<Vec<_>>(),
            [17, 23]
        );
    }

    #[test]
    fn clear_and_drop_destroy_each_owned_item_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        #[derive(Debug)]
        struct Tracked(Arc<AtomicUsize>);
        impl Drop for Tracked {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        {
            let mut buffer = Buffer::try_with_capacity(2).unwrap();
            buffer.try_push(Tracked(Arc::clone(&drops))).unwrap();
            buffer.try_push(Tracked(Arc::clone(&drops))).unwrap();
            buffer.clear();
            assert_eq!(drops.load(Ordering::SeqCst), 2);
            buffer.try_push(Tracked(Arc::clone(&drops))).unwrap();
        }
        assert_eq!(drops.load(Ordering::SeqCst), 3);
    }
}
