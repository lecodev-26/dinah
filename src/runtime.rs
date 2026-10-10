//! Minimal Runtime execution primitives.
//!
//! This module starts with a synchronous executor. It deliberately does not
//! create threads, schedule multiple tasks, or capture panics.

mod executor;
mod queue;
mod scheduler;

pub use executor::Executor;
pub use queue::{QueuePushError, TaskQueue};
pub use scheduler::{ScheduleError, Scheduler};
