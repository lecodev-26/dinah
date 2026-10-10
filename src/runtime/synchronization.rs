use core::fmt;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, PoisonError};

/// Cloneable shared state protected by a standard-library mutex.
///
/// Clones refer to the same value. Lock acquisition serializes access across
/// threads. A guard must be held only for the smallest necessary state update;
/// callers should not run arbitrary task code while holding it. If a thread
/// panics while holding the lock, subsequent acquisitions return
/// [`SynchronizationError::Poisoned`] rather than silently ignoring possible
/// invariant damage. This type does not provide lock-free access or fairness.
#[derive(Debug)]
pub struct Shared<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Failure to acquire shared state safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynchronizationError {
    /// A previous panic occurred while the mutex was locked.
    Poisoned,
}

impl fmt::Display for SynchronizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poisoned => f.write_str("shared state lock is poisoned"),
        }
    }
}

impl std::error::Error for SynchronizationError {}

impl<T> Shared<T> {
    /// Wraps a value in a mutex-backed shared owner.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    /// Acquires exclusive access to the shared value.
    ///
    /// The returned guard releases the lock when dropped. Poisoning is reported
    /// as an explicit error; this API deliberately does not recover the guard
    /// because the protected value's invariants are unknown to the wrapper.
    pub fn lock(&self) -> Result<MutexGuard<'_, T>, SynchronizationError> {
        self.inner.lock().map_err(|_| SynchronizationError::Poisoned)
    }

    /// Returns the number of strong shared owners, primarily for diagnostics.
    #[must_use]
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl<T> From<LockResult<MutexGuard<'_, T>>> for SynchronizationError {
    fn from(_: LockResult<MutexGuard<'_, T>>) -> Self {
        Self::Poisoned
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for SynchronizationError {
    fn from(_: PoisonError<MutexGuard<'_, T>>) -> Self {
        Self::Poisoned
    }
}

#[cfg(test)]
mod tests {
    use super::{Shared, SynchronizationError};

    #[test]
    fn clones_share_the_same_value() {
        let shared = Shared::new(10);
        let clone = shared.clone();
        *clone.lock().expect("lock should be healthy") += 5;
        assert_eq!(*shared.lock().expect("lock should be healthy"), 15);
        assert_eq!(shared.strong_count(), 2);
    }

    #[test]
    fn serializes_concurrent_updates() {
        let shared = Shared::new(0_u64);
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let shared = shared.clone();
                std::thread::spawn(move || {
                    for _ in 0..1_000 {
                        *shared.lock().expect("lock should be healthy") += 1;
                    }
                })
            })
            .collect();

        for thread in threads {
            thread.join().expect("worker should not panic");
        }
        assert_eq!(*shared.lock().expect("lock should be healthy"), 8_000);
    }

    #[test]
    fn reports_poisoned_lock_instead_of_silently_recovering() {
        let shared = Shared::new(0);
        let clone = shared.clone();
        let _ = std::thread::spawn(move || {
            let _guard = clone.lock().expect("initial lock should be healthy");
            panic!("poison the test mutex");
        })
        .join();

        assert_eq!(shared.lock().unwrap_err(), SynchronizationError::Poisoned);
        assert_eq!(
            SynchronizationError::Poisoned.to_string(),
            "shared state lock is poisoned"
        );
    }
}
