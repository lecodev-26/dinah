use core::fmt;

/// Result type used by Core operations.
///
/// Prefer a more specific error type when callers need to distinguish the
/// precise reason an operation failed.
pub type CoreResult<T> = Result<T, CoreError>;

/// Coarse errors shared by the foundational Core APIs.
///
/// This enum intentionally groups detailed configuration errors into
/// [`CoreError::InvalidConfiguration`]; APIs that need the field-level reason
/// should return [`super::CoreConfigError`] directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreError {
    /// A configuration value did not satisfy its documented constraints.
    InvalidConfiguration,
    /// An operation is not valid for the current lifecycle state.
    InvalidState,
    /// A numeric identifier was outside the supported range.
    InvalidId,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration => f.write_str("invalid core configuration"),
            Self::InvalidState => f.write_str("operation is invalid for the current state"),
            Self::InvalidId => f.write_str("invalid core identifier"),
        }
    }
}

impl std::error::Error for CoreError {}
