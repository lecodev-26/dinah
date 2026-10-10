use core::fmt;

/// A fixed-logical-capacity, contiguous buffer that owns its elements.
///
/// The backing `Vec` reserves the configured maximum at construction, so
/// successful construction makes pushes up to that limit avoid growth-related
/// reallocations. The allocator may reserve more than requested. Capacity is
/// an element limit, not a byte budget or a residency guarantee.
///
/// This type is not internally synchronized. Mutations require exclusive
/// access; [`Self::as_slice`] lends read-only access to initialized elements.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl<T> Buffer<T> {
    /// Creates an empty buffer with a fixed maximum element capacity.
    ///
    /// Capacity zero is valid. Reservation failure is reported rather than
    /// handled by an infallible allocation path.
    pub fn try_with_capacity(max_capacity: usize) -> Result<Self, BufferCreateError> {
        let mut items = Vec::new();
        items.try_reserve_exact(max_capacity)
            .map_err(|_| BufferCreateError::AllocationFailed)?;
        Ok(Self { items, max_capacity })
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

    /// Removes and returns the last item, if present.
    pub fn pop(&mut self) -> Option<T> { self.items.pop() }

    /// Removes all items while retaining the reserved backing storage.
    pub fn clear(&mut self) { self.items.clear(); }

    /// Returns the initialized elements as a contiguous read-only slice.
    #[must_use]
    pub fn as_slice(&self) -> &[T] { self.items.as_slice() }

    /// Returns the number of initialized elements.
    #[must_use]
    pub fn len(&self) -> usize { self.items.len() }

    /// Returns whether the buffer contains no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool { self.items.is_empty() }

    /// Returns the configured maximum number of elements.
    #[must_use]
    pub const fn capacity(&self) -> usize { self.max_capacity }

    /// Returns whether the configured maximum has been reached.
    #[must_use]
    pub fn is_full(&self) -> bool { self.len() == self.capacity() }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, BufferCreateError, BufferPushError};

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
    fn full_buffer_returns_item_without_changing_contents() {
        let mut buffer = Buffer::try_with_capacity(1).unwrap();
        buffer.try_push("kept").unwrap();
        assert_eq!(buffer.try_push("returned"), Err(BufferPushError::Full("returned")));
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
        assert_eq!(Buffer::<u8>::try_with_capacity(usize::MAX).unwrap_err(), BufferCreateError::AllocationFailed);
    }
}
