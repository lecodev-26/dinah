//! dinah: a Rust-first foundation for runtime, compute, and data-oriented systems.
//!
//! The Core module provides small, dependency-free primitives shared by future
//! subsystems. It intentionally avoids runtime, I/O, and platform assumptions.

#![forbid(unsafe_code)]

pub mod core;

/// Returns the crate's current package name.
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
