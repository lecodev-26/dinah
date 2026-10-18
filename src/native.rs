//! Native platform capability contracts and conservative standard-library backend.
//!
//! This module intentionally contains no unsafe code and does not promise direct
//! hardware access. Platform-specific extensions must remain behind these traits.
use std::time::{Duration, Instant};

/// Stable, dependency-free description of the host capabilities exposed by this crate.
pub trait Hal {
    /// Returns a compile-time target operating-system identifier.
    fn operating_system(&self) -> &'static str;
    /// Returns a compile-time target architecture identifier.
    fn architecture(&self) -> &'static str;
    /// Returns elapsed monotonic time since `origin`, saturating to zero only if the
    /// underlying platform reports an unexpected ordering.
    fn elapsed_since(&self, origin: Instant) -> Duration;
}

/// Platform service contract separated from any direct OS API or hardware access.
pub trait Platform: Hal {
    /// Returns whether the current build target is Unix-like.
    fn is_unix(&self) -> bool;
    /// Returns the executable's process identifier where the standard library supports it.
    fn process_id(&self) -> u32;
}

/// Standard-library implementation suitable as the default capability provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostPlatform;
impl HostPlatform {
    /// Creates a host capability provider.
    pub const fn new() -> Self {
        Self
    }
}
impl Hal for HostPlatform {
    fn operating_system(&self) -> &'static str {
        std::env::consts::OS
    }
    fn architecture(&self) -> &'static str {
        std::env::consts::ARCH
    }
    fn elapsed_since(&self, origin: Instant) -> Duration {
        origin.elapsed()
    }
}
impl Platform for HostPlatform {
    fn is_unix(&self) -> bool {
        cfg!(unix)
    }
    fn process_id(&self) -> u32 {
        std::process::id()
    }
}

/// Unix/POSIX-family adapter built on stable Rust standard-library facilities.
///
/// This is a conservative platform-information layer, not a direct libc binding.
#[cfg(unix)]
pub mod posix {
    use super::{Hal, HostPlatform, Platform};
    use std::time::{Duration, Instant};

    /// POSIX-family host provider. It deliberately delegates to safe std APIs.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct PosixPlatform(HostPlatform);
    impl PosixPlatform {
        /// Creates a POSIX-family capability provider.
        pub const fn new() -> Self {
            Self(HostPlatform::new())
        }
    }
    impl Hal for PosixPlatform {
        fn operating_system(&self) -> &'static str {
            self.0.operating_system()
        }
        fn architecture(&self) -> &'static str {
            self.0.architecture()
        }
        fn elapsed_since(&self, origin: Instant) -> Duration {
            self.0.elapsed_since(origin)
        }
    }
    impl Platform for PosixPlatform {
        fn is_unix(&self) -> bool {
            true
        }
        fn process_id(&self) -> u32 {
            self.0.process_id()
        }
    }
}

/// Minimal C ABI query surface. Functions accept/return only fixed-width scalar values.
/// No pointer ownership, allocation, callbacks, or borrowed strings cross the boundary.
#[allow(unsafe_code)] // Only exported symbol names; this module contains no unsafe blocks or pointer APIs.
pub mod ffi {
    /// Returns the integer ABI version for consumers to check before using this interface.
    #[no_mangle]
    pub extern "C" fn dinah_abi_version() -> u32 {
        1
    }

    /// Returns the target feature flags supported by this minimal ABI. Bit 0 means the
    /// scalar metadata queries are available; all other bits are currently reserved.
    #[no_mangle]
    pub extern "C" fn dinah_abi_features() -> u64 {
        1
    }

    /// Returns the numeric target family: 1 = Unix, 2 = Windows, 0 = other.
    #[no_mangle]
    pub extern "C" fn dinah_abi_platform_family() -> u32 {
        if cfg!(unix) {
            1
        } else if cfg!(windows) {
            2
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Hal, HostPlatform, Platform};
    use std::time::{Duration, Instant};

    #[test]
    fn host_provider_reports_target_and_process() {
        let host = HostPlatform::new();
        assert!(!host.operating_system().is_empty());
        assert!(!host.architecture().is_empty());
        assert!(host.process_id() > 0);
        assert_eq!(host.is_unix(), cfg!(unix));
        assert!(
            host.elapsed_since(Instant::now() - Duration::from_millis(1))
                >= Duration::from_millis(1)
        );
    }

    #[test]
    fn c_abi_scalar_queries_are_versioned() {
        assert_eq!(super::ffi::dinah_abi_version(), 1);
        assert_eq!(super::ffi::dinah_abi_features() & 1, 1);
        assert!(super::ffi::dinah_abi_platform_family() <= 2);
    }

    #[cfg(unix)]
    #[test]
    fn posix_provider_matches_host_metadata() {
        use super::posix::PosixPlatform;
        let host = HostPlatform::new();
        let posix = PosixPlatform::new();
        assert_eq!(posix.operating_system(), host.operating_system());
        assert_eq!(posix.architecture(), host.architecture());
        assert!(posix.is_unix());
    }
}
