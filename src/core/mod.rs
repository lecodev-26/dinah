//! Foundational types and validated configuration for dinah.
//!
//! These APIs are deliberately minimal and dependency-free. Higher-level
//! subsystems should build on them rather than introduce cross-layer coupling.

mod config;
mod error;
mod id;
mod state;

pub use config::{CoreConfig, CoreConfigError};
pub use error::{CoreError, CoreResult};
pub use id::{CoreId, IdError};
pub use state::LifecycleState;

/// Shared version of the Core API contract.
pub const CORE_API_VERSION: u16 = 1;

#[cfg(test)]
mod tests;
