# Native / Hardware foundation (points 157–159)

## Scope and safety boundary

`dinah::native` introduces a small Hardware Abstraction Layer (HAL) contract and platform-service contract without claiming direct device access. `Hal` exposes target OS, target architecture and elapsed monotonic time; `Platform` adds Unix-family detection and process ID. `HostPlatform` is the portable standard-library implementation. On Unix targets, `native::posix::PosixPlatform` provides the same metadata through safe Rust standard-library facilities.

This first POSIX layer is intentionally conservative: it does not call libc directly, manage file descriptors, change signals, use `mmap`, inspect devices, or assume Linux-only syscalls. `cfg(unix)` covers a family of targets and is not a Linux-specific guarantee. Future OS bindings require target-specific CI and a reviewed safety boundary.

## API contracts

- OS and architecture values come from Rust's compile-time `std::env::consts` constants.
- Elapsed time uses `Instant`, which is monotonic but has no portable wall-clock epoch.
- Process ID is a diagnostic identifier, not a stable identity or security credential.
- Platform family is a coarse target classification, not proof of a particular kernel feature.
- The traits are synchronous and do not allocate, start workers, or access hardware.

## Validation and limitations

Unit tests check non-empty target metadata, process ID, monotonic elapsed time and equivalence between the Unix adapter and host provider. These are contract tests, not a complete platform matrix. The POSIX adapter has no direct libc dependency and makes no claim about page size, CPU topology, device access, affinity, memory mapping or kernel interfaces.
