use core::fmt;

/// Executes one task synchronously on the calling thread.
///
/// `Executor` is the initial execution boundary for Runtime. It does not spawn
/// threads, maintain a queue, retry work, or capture panics. The caller retains
/// control until the closure returns or unwinds. Because the task is accepted as
/// `FnOnce`, this API can invoke the supplied closure at most once.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Executor;

impl Executor {
    /// Creates an executor that runs tasks on the calling thread.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Runs a one-shot task and returns its result without erasing its error.
    ///
    /// The closure is called exactly once during a normal return from this
    /// method. If it returns `Err`, that error is passed through unchanged. A
    /// panic is not caught and unwinds to the caller.
    pub fn execute<T, E, F>(&self, task: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        task()
    }
}

impl fmt::Display for Executor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("synchronous executor")
    }
}

#[cfg(test)]
mod tests {
    use super::Executor;
    use core::cell::Cell;

    #[test]
    fn executes_task_and_returns_value() {
        let executor = Executor::new();
        assert_eq!(executor.execute(|| Ok::<_, &'static str>(42)), Ok(42));
    }

    #[test]
    fn invokes_one_shot_task_once() {
        let executor = Executor::new();
        let calls = Cell::new(0);

        let result = executor.execute(|| {
            calls.set(calls.get() + 1);
            Ok::<_, &'static str>(())
        });

        assert_eq!(result, Ok(()));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn preserves_task_error() {
        let executor = Executor::new();
        let result = executor.execute(|| Err::<(), _>("task-specific failure"));
        assert_eq!(result, Err("task-specific failure"));
    }

    #[test]
    fn display_describes_execution_mode() {
        assert_eq!(Executor::new().to_string(), "synchronous executor");
    }
}
