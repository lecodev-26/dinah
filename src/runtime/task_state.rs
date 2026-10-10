use core::fmt;

/// Observable lifecycle state for one Runtime task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskState {
    /// Returns whether the requested lifecycle transition is permitted.
    #[must_use]
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Running)
                | (Self::Pending, Self::Cancelled)
                | (Self::Running, Self::Completed)
                | (Self::Running, Self::Failed)
                | (Self::Running, Self::Cancelled)
        )
    }

    /// Applies a valid transition or leaves the state unchanged on error.
    pub fn transition_to(&mut self, next: Self) -> Result<(), InvalidTaskTransition> {
        if !self.can_transition_to(next) {
            return Err(InvalidTaskTransition {
                from: *self,
                to: next,
            });
        }
        *self = next;
        Ok(())
    }

    /// Returns whether this state is terminal.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// A rejected transition, retaining its source and destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTaskTransition {
    from: TaskState,
    to: TaskState,
}

impl InvalidTaskTransition {
    #[must_use]
    pub const fn from(self) -> TaskState {
        self.from
    }
    #[must_use]
    pub const fn to(self) -> TaskState {
        self.to
    }
}

impl fmt::Display for InvalidTaskTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid task state transition: {:?} -> {:?}",
            self.from, self.to
        )
    }
}
impl std::error::Error for InvalidTaskTransition {}

#[cfg(test)]
mod tests {
    use super::TaskState;

    #[test]
    fn allows_only_documented_transitions() {
        for (from, to) in [
            (TaskState::Pending, TaskState::Running),
            (TaskState::Pending, TaskState::Cancelled),
            (TaskState::Running, TaskState::Completed),
            (TaskState::Running, TaskState::Failed),
            (TaskState::Running, TaskState::Cancelled),
        ] {
            assert!(from.can_transition_to(to));
        }
        assert!(!TaskState::Pending.can_transition_to(TaskState::Completed));
        assert!(!TaskState::Completed.can_transition_to(TaskState::Running));
        assert!(!TaskState::Running.can_transition_to(TaskState::Running));
    }

    #[test]
    fn invalid_transition_preserves_terminal_state() {
        let mut state = TaskState::Completed;
        let error = state.transition_to(TaskState::Running).unwrap_err();
        assert_eq!(error.from(), TaskState::Completed);
        assert_eq!(error.to(), TaskState::Running);
        assert_eq!(state, TaskState::Completed);
        assert!(state.is_terminal());
    }

    #[test]
    fn valid_transition_updates_state() {
        let mut state = TaskState::Pending;
        state.transition_to(TaskState::Running).unwrap();
        state.transition_to(TaskState::Failed).unwrap();
        assert_eq!(state, TaskState::Failed);
    }
}
