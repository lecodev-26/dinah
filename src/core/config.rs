use super::CoreError;
use core::fmt;

/// Validated limits for foundational dinah subsystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreConfig {
    max_workers: usize,
    max_batch_size: usize,
}

/// Configuration validation failure, with the field that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreConfigError {
    /// Worker count must be at least one.
    ZeroWorkers,
    /// Batch size must be at least one.
    ZeroBatchSize,
}

impl CoreConfig {
    /// Creates configuration after validating all limits.
    pub const fn new(max_workers: usize, max_batch_size: usize) -> Result<Self, CoreConfigError> {
        if max_workers == 0 {
            return Err(CoreConfigError::ZeroWorkers);
        }
        if max_batch_size == 0 {
            return Err(CoreConfigError::ZeroBatchSize);
        }
        Ok(Self { max_workers, max_batch_size })
    }

    /// Returns a conservative configuration suitable for a single worker.
    pub const fn default_config() -> Self {
        Self { max_workers: 1, max_batch_size: 64 }
    }

    pub const fn max_workers(self) -> usize { self.max_workers }
    pub const fn max_batch_size(self) -> usize { self.max_batch_size }

    /// Checks whether this configuration satisfies Core's invariants.
    pub const fn validate(self) -> Result<(), CoreConfigError> {
        if self.max_workers == 0 { return Err(CoreConfigError::ZeroWorkers); }
        if self.max_batch_size == 0 { return Err(CoreConfigError::ZeroBatchSize); }
        Ok(())
    }
}

impl Default for CoreConfig {
    fn default() -> Self { Self::default_config() }
}

impl fmt::Display for CoreConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroWorkers => f.write_str("max_workers must be greater than zero"),
            Self::ZeroBatchSize => f.write_str("max_batch_size must be greater than zero"),
        }
    }
}

impl std::error::Error for CoreConfigError {}

impl From<CoreConfigError> for CoreError {
    fn from(_: CoreConfigError) -> Self { Self::InvalidConfiguration }
}
