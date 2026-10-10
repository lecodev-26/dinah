use core::fmt;

/// Describes why a Runtime task failed without erasing its concrete error.
///
/// This wrapper classifies an error value supplied by the caller; it does not
/// attempt to catch panics or infer failure causes from error text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskFailure<E> {
    kind: TaskFailureKind,
    error: E,
}

/// Stable category for a task-level failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskFailureKind {
    /// The task returned an ordinary error.
    ReturnedError,
}

impl<E> TaskFailure<E> {
    /// Records an error returned by task code.
    #[must_use]
    pub const fn returned(error: E) -> Self {
        Self {
            kind: TaskFailureKind::ReturnedError,
            error,
        }
    }

    /// Returns the stable classification of this failure.
    #[must_use]
    pub const fn kind(&self) -> TaskFailureKind {
        self.kind
    }

    /// Borrows the original task error.
    #[must_use]
    pub const fn error(&self) -> &E {
        &self.error
    }

    /// Consumes the wrapper and returns the original task error.
    #[must_use]
    pub fn into_error(self) -> E {
        self.error
    }

    /// Maps the stored error while preserving its classification.
    pub fn map_error<F, M>(self, map: M) -> TaskFailure<F>
    where
        M: FnOnce(E) -> F,
    {
        TaskFailure {
            kind: self.kind,
            error: map(self.error),
        }
    }
}

impl<E: fmt::Display> fmt::Display for TaskFailure<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TaskFailureKind::ReturnedError => write!(f, "task returned an error: {}", self.error),
        }
    }
}

impl<E: fmt::Debug + fmt::Display + 'static> std::error::Error for TaskFailure<E> {}

#[cfg(test)]
mod tests {
    use super::{TaskFailure, TaskFailureKind};

    #[test]
    fn preserves_original_error_and_classification() {
        let failure = TaskFailure::returned(String::from("disk unavailable"));
        assert_eq!(failure.kind(), TaskFailureKind::ReturnedError);
        assert_eq!(failure.error(), "disk unavailable");
        assert_eq!(failure.to_string(), "task returned an error: disk unavailable");
        assert_eq!(failure.into_error(), "disk unavailable");
    }

    #[test]
    fn maps_error_without_changing_classification() {
        let failure = TaskFailure::returned(17_u8).map_error(|code| format!("code={code}"));
        assert_eq!(failure.kind(), TaskFailureKind::ReturnedError);
        assert_eq!(failure.error(), "code=17");
    }
}
