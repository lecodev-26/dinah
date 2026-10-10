use core::fmt;

/// Result type used by Core operations.
pub type CoreResult<T> = Result<T, CoreError>;

/// Errors shared by the foundational Core APIs.
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
