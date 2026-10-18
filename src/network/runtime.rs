//! Explicit bridge for dispatching network-related work through Runtime.
use crate::runtime::{ScheduleError, Scheduler, TaskQueue};
use core::num::NonZeroUsize;

/// Bounded queue of caller-created one-shot network work items.
///
/// Enqueueing does not execute work. `dispatch_next` runs one item synchronously
/// through `Scheduler`; there are no hidden threads, sockets, or polling loops.
pub struct NetworkWorkQueue<E> {
    queue: TaskQueue<Box<dyn FnOnce() -> Result<(), E>>>,
}
impl<E> NetworkWorkQueue<E> {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            queue: TaskQueue::new(capacity),
        }
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn enqueue(
        &mut self,
        task: Box<dyn FnOnce() -> Result<(), E>>,
    ) -> Result<(), Box<dyn FnOnce() -> Result<(), E>>> {
        self.queue.try_push(task).map_err(|error| match error {
            crate::runtime::QueuePushError::Full(task) => task,
        })
    }
    pub fn dispatch_next(
        &mut self,
        scheduler: &Scheduler,
    ) -> Option<Result<Result<(), E>, ScheduleError>> {
        self.queue.pop().map(|task| scheduler.schedule(task))
    }
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }
}
