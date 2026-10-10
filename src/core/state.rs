/// Minimal lifecycle states shared by future managed resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    /// Resource has been created but is not active.
    Ready,
    /// Resource is active and may accept work.
    Running,
    /// Resource is stopping and should not accept new work.
    Stopping,
    /// Resource has stopped and is terminal.
    Stopped,
}

impl LifecycleState {
    /// Returns whether a transition is permitted by the Core lifecycle contract.
    ///
    /// This is a pure check: it does not change a resource state, coordinate
    /// concurrent callers, or imply that a resource has completed a transition.
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Ready, Self::Running)
                | (Self::Ready, Self::Stopped)
                | (Self::Running, Self::Stopping)
                | (Self::Stopping, Self::Stopped)
        )
    }
}
