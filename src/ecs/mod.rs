//! Entity-component-system foundations.
//!
//! The ECS is introduced incrementally. Entity handles, the component type
//! contract, and entity lifetime tracking are defined before component storage.

mod access;
mod component;
mod entity;
#[allow(dead_code)] // Integrated into World in a later ECS step.
mod storage;
mod world;

pub use access::{AccessMode, SystemAccess};
pub use component::Component;
pub use entity::Entity;
pub use world::{World, WorldError};
