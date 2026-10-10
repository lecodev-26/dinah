//! Foundational types and validated configuration for dinah.
//!
//! These APIs are deliberately minimal and dependency-free. Higher-level
//! subsystems should build on them rather than introduce cross-layer coupling.

mod config;
mod error;
mod id;
mod state;
mod traits;

pub use config::{CoreConfig, CoreConfigError};
pub use error::{CoreError, CoreResult};
pub use id::{CoreId, IdError};
pub use state::LifecycleState;
pub use traits::{HasCoreId, Validate};

/// Shared version of the Core API contract.
pub const CORE_API_VERSION: u16 = 1;

#[cfg(test)]
mod tests;

/// Core invariants relied on by every subsystem:
/// - IDs represented by `CoreId` are never zero.
/// - `CoreConfig` limits are non-zero.
/// - Lifecycle transitions must be explicitly permitted by `can_transition_to`.
/// - Core primitives do not perform I/O or spawn threads as a side effect.
pub const CORE_INVARIANTS: [&str; 4] = [
    "CoreId is non-zero",
    "configuration limits are non-zero",
    "lifecycle transitions are explicit",
    "Core primitives have no I/O or thread-spawning side effects",
];
