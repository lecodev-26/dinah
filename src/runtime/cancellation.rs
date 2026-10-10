use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// Cloneable cooperative cancellation signal for a task or group of tasks.
///
/// Requesting cancellation only sets a flag; it never interrupts running code.
/// Task code must check the token at safe points and decide how to stop and
/// release its resources. An observed request is not, by itself, proof that the
/// task has stopped; the caller must still observe task completion.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    requested: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a cancellation token with no request pending.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation. Repeated requests are safe and idempotent.
    ///
    /// Returns `true` only for the call that changes the flag from false to
    /// true; `false` means cancellation had already been requested.
    pub fn request_cancellation(&self) -> bool {
        !self.requested.swap(true, Ordering::AcqRel)
    }

    /// Reports whether cancellation has been requested.
    #[must_use]
    pub fn is_cancellation_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::CancellationToken;

    #[test]
    fn clones_observe_the_same_request() {
        let token = CancellationToken::new();
        let worker_token = token.clone();
        assert!(!token.is_cancellation_requested());
        assert!(worker_token.request_cancellation());
        assert!(token.is_cancellation_requested());
        assert!(!token.request_cancellation());
    }

    #[test]
    fn concurrent_requests_are_idempotent_and_visible() {
        let token = CancellationToken::new();
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let token = token.clone();
                std::thread::spawn(move || token.request_cancellation())
            })
            .collect();
        let first_request_count = threads
            .into_iter()
            .filter(|thread| thread.join().expect("request thread should finish"))
            .count();
        assert_eq!(first_request_count, 1);
        assert!(token.is_cancellation_requested());
    }
}
