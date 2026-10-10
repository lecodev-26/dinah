use super::{CoreConfig, CoreConfigError, CoreId};

/// Common contract for values that can validate their own invariants.
pub trait Validate {
    /// The precise reason validation failed.
    type Error;

    /// Returns `Ok(())` only when the value satisfies its contract.
    fn validate(&self) -> Result<(), Self::Error>;
}

impl Validate for CoreConfig {
    type Error = CoreConfigError;

    fn validate(&self) -> Result<(), Self::Error> {
        (*self).validate()
    }
}

/// Contract for values that expose a stable, non-zero Core identifier.
pub trait HasCoreId {
    fn core_id(&self) -> CoreId;
}
