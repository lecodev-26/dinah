//! Minimal Runtime execution primitives.
//!
//! This module starts with a synchronous executor. It deliberately does not
//! create threads, schedule multiple tasks, or capture panics.

mod executor;
mod scheduler;

pub use executor::Executor;
pub use scheduler::{ScheduleError, Scheduler};
