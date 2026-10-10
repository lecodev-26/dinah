//! Safe, bounded memory primitives.
//!
//! The initial API uses standard-library containers and exposes no raw pointers
//! or custom allocator hooks. Buffers own their elements and lend read-only
//! slices; synchronization is the caller responsibility.

mod buffer;

pub use buffer::{Buffer, BufferCreateError, BufferPushError};
