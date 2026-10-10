//! Minimal Runtime execution primitives.
//!
//! This module provides synchronous execution, a bounded FIFO queue, and
//! queue-backed sequential dispatch. It deliberately creates no worker threads
//! and does not capture panics.

mod executor;
mod queue;
mod scheduler;

pub use executor::Executor;
pub use queue::{QueuePushError, TaskQueue};
pub use scheduler::{ScheduleError, Scheduler};
