use super::{CoreConfig, CoreConfigError, CoreId};

/// Common contract for values that can validate their own invariants.
///
/// Implementations should be deterministic and side-effect-free. Validation
/// reports whether the value satisfies its documented contract; it does not
/// promise that later operations will succeed if external conditions change.
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
///
/// Implementors must return the same identifier for the lifetime of the value.
/// The trait does not allocate IDs, establish global uniqueness, or imply that
/// the implementing value is registered with a runtime.
pub trait HasCoreId {
    /// Returns the stable identifier exposed by this value.
    fn core_id(&self) -> CoreId;
}
