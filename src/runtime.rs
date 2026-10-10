//! Minimal Runtime execution primitives.
//!
//! This module provides synchronous execution, a bounded FIFO queue, sequential
//! dispatch, and bounded parallel execution using scoped workers reused within
//! each parallel call. Cancellation, metrics, tracing, task-state, and shared-state
//! primitives are opt-in and caller-managed, not automatically wired into tasks.
//! Task panics propagate to callers after the current parallel batch is joined.

mod cancellation;
mod executor;
mod metrics;
mod outcome;
mod queue;
mod scheduler;
mod synchronization;
mod task_state;
mod tracing;

pub use cancellation::CancellationToken;
pub use executor::Executor;
pub use metrics::{MetricsSnapshot, RuntimeMetrics};
pub use outcome::{TaskFailure, TaskFailureKind};
pub use queue::{QueuePushError, TaskQueue};
pub use scheduler::{ScheduleError, Scheduler};
pub use synchronization::{Shared, SynchronizationError};
pub use task_state::{InvalidTaskTransition, TaskState};
pub use tracing::{TraceBuffer, TraceEvent, TraceRecordError};
