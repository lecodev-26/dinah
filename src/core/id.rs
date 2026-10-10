use core::fmt;

/// Non-zero identifier shared by Core-managed objects.
///
/// Zero is reserved as an invalid/uninitialized sentinel and cannot be
/// represented by this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreId(u64);

/// Failure to construct a valid [`CoreId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdError;

impl CoreId {
    pub const fn new(value: u64) -> Result<Self, IdError> {
        if value == 0 { Err(IdError) } else { Ok(Self(value)) }
    }

    pub const fn get(self) -> u64 { self.0 }
}

impl fmt::Display for CoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("core identifier must be non-zero")
    }
}

impl std::error::Error for IdError {}
