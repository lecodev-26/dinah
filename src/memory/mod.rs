//! Safe, bounded memory primitives.
//!
//! The initial API uses standard-library containers and exposes no raw pointers
//! or custom allocator hooks. Buffers own their elements and lend read-only
//! slices; synchronization is the caller responsibility. Ownership is unique
//! by default: buffers are moved rather than implicitly cloned, and borrowed
//! views cannot outlive their owner.

mod buffer;
mod soa;

pub use buffer::{Buffer, BufferCreateError, BufferExtendError, BufferPushError};
pub use soa::{SoaBuffer, SoaCreateError, SoaExtendError, SoaPushError};
