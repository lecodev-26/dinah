//! DINAH: a Rust-first foundation for runtime, compute, and data-oriented systems.
//!
//! The crate is intentionally minimal at project bootstrap. Public APIs will be
//! introduced incrementally with tests and documentation as roadmap items advance.

#![forbid(unsafe_code)]

/// Returns the crate's current package name.
///
/// This tiny smoke-testable function is temporary bootstrap scaffolding, not a
/// claim that the Core roadmap phase is complete.
pub const fn crate_name() -> &'static str {
    "dinah"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_expected_crate_name() {
        assert_eq!(crate_name(), "dinah");
    }
}
