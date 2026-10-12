//! Entity-component-system foundations.
//!
//! The ECS is introduced incrementally. Entity handles, the component type
//! contract, and entity lifetime tracking are defined before component storage.

mod component;
mod entity;
mod world;

pub use component::Component;
pub use entity::Entity;
pub use world::{World, WorldError};
