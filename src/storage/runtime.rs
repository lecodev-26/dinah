//! Explicit bridge for applying bounded storage mutations through Runtime.
use super::{Storage, StorageError};
use crate::runtime::{ScheduleError, Scheduler, TaskQueue};
use core::num::NonZeroUsize;

/// An owned mutation waiting for explicit dispatch.
#[derive(Debug)]
pub enum StorageCommand {
    /// Insert or replace a key.
    Put { key: Vec<u8>, value: Vec<u8> },
    /// Delete a key if present.
    Delete { key: Vec<u8> },
}

/// Bounded FIFO of storage mutations. Enqueueing never touches the store.
pub struct StorageWorkQueue {
    queue: TaskQueue<StorageCommand>,
}
impl StorageWorkQueue {
    /// Creates a queue with a non-zero fixed capacity.
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            queue: TaskQueue::new(capacity),
        }
    }
    /// Number of queued commands.
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    /// Maximum number of queued commands.
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }
    /// Adds a mutation or returns ownership of the command if the queue is full.
    pub fn enqueue(&mut self, command: StorageCommand) -> Result<(), StorageCommand> {
        self.queue.try_push(command).map_err(|error| match error {
            crate::runtime::QueuePushError::Full(command) => command,
        })
    }
    /// Applies one FIFO mutation synchronously via Runtime; no worker is spawned.
    pub fn dispatch_next(
        &mut self,
        scheduler: &Scheduler,
        store: &mut dyn Storage,
    ) -> Option<Result<Result<(), StorageError>, ScheduleError>> {
        self.queue.pop().map(|command| {
            scheduler.schedule(|| match command {
                StorageCommand::Put { key, value } => store.put(&key, &value),
                StorageCommand::Delete { key } => store.delete(&key).map(|_| ()),
            })
        })
    }
}
