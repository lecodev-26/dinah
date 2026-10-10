use core::num::NonZeroUsize;
use std::collections::VecDeque;

/// A bounded FIFO queue for pending Runtime work items.
///
/// This queue only stores values; it does not execute them, synchronize across
/// threads, or define scheduler policy. When full, [`TaskQueue::try_push`]
/// returns ownership of the rejected item to the caller so work is never
/// silently discarded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskQueue<T> {
    items: VecDeque<T>,
    capacity: NonZeroUsize,
}

/// Failure to push an item into a [`TaskQueue`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueuePushError<T> {
    /// The queue has reached its configured capacity; contains the rejected item.
    Full(T),
}

impl<T> TaskQueue<T> {
    /// Creates an empty queue with a strictly positive capacity.
    #[must_use]
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity.get()),
            capacity,
        }
    }

    /// Attempts to append an item at the back of the queue.
    ///
    /// Returns [`QueuePushError::Full`] with the original item when no capacity
    /// remains. A successful push preserves FIFO order.
    pub fn try_push(&mut self, item: T) -> Result<(), QueuePushError<T>> {
        if self.is_full() {
            return Err(QueuePushError::Full(item));
        }
        self.items.push_back(item);
        Ok(())
    }

    /// Removes and returns the oldest queued item, if any.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    /// Returns the number of queued items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the queue contains no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the configured maximum number of queued items.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity.get()
    }

    /// Returns whether the queue has reached its configured capacity.
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::{QueuePushError, TaskQueue};
    use core::num::NonZeroUsize;

    fn queue<T>(capacity: usize) -> TaskQueue<T> {
        TaskQueue::new(NonZeroUsize::new(capacity).expect("test capacity is positive"))
    }

    #[test]
    fn starts_empty_with_configured_capacity() {
        let queue = queue::<u8>(2);
        assert!(queue.is_empty());
        assert!(!queue.is_full());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.capacity(), 2);
    }

    #[test]
    fn preserves_fifo_order() {
        let mut queue = queue(3);
        assert_eq!(queue.try_push(10), Ok(()));
        assert_eq!(queue.try_push(20), Ok(()));
        assert_eq!(queue.try_push(30), Ok(()));
        assert!(queue.is_full());
        assert_eq!(queue.pop(), Some(10));
        assert_eq!(queue.pop(), Some(20));
        assert_eq!(queue.pop(), Some(30));
        assert_eq!(queue.pop(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn rejects_full_queue_without_losing_item() {
        let mut queue = queue(1);
        assert_eq!(queue.try_push("accepted"), Ok(()));
        assert_eq!(
            queue.try_push("rejected"),
            Err(QueuePushError::Full("rejected"))
        );
        assert_eq!(queue.pop(), Some("accepted"));
    }

    #[test]
    fn capacity_one_can_be_reused_after_pop() {
        let mut queue = queue(1);
        assert_eq!(queue.try_push(1), Ok(()));
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.try_push(2), Ok(()));
        assert_eq!(queue.pop(), Some(2));
    }
}
