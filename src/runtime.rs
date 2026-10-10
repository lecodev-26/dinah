//! Minimal Runtime execution primitives.
//!
//! This module provides synchronous execution, a bounded FIFO queue, sequential
//! dispatch, and bounded parallel batches using temporary scoped threads. Panics
//! are not captured or converted into task errors.

mod executor;
mod queue;
mod scheduler;

pub use executor::Executor;
pub use queue::{QueuePushError, TaskQueue};
pub use scheduler::{ScheduleError, Scheduler};
