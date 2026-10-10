//! Minimal Runtime execution primitives.
//!
//! This module provides synchronous execution, a bounded FIFO queue, sequential
//! dispatch, bounded parallel batches using temporary scoped threads, and opt-in
//! shared state protected by a mutex. Panics are not converted into task errors.

mod cancellation;
mod executor;
mod metrics;
mod outcome;
mod queue;
mod scheduler;
mod synchronization;
mod task_state;

pub use cancellation::CancellationToken;
pub use executor::Executor;
pub use metrics::{MetricsSnapshot, RuntimeMetrics};
pub use outcome::{TaskFailure, TaskFailureKind};
pub use queue::{QueuePushError, TaskQueue};
pub use scheduler::{ScheduleError, Scheduler};
pub use synchronization::{Shared, SynchronizationError};
pub use task_state::{InvalidTaskTransition, TaskState};
