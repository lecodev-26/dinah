//! Entity-component-system foundations.
//!
//! The ECS is introduced incrementally. Entity handles and the component type
//! contract are defined first; storage and world ownership follow later.

mod component;
mod entity;

pub use component::Component;
pub use entity::Entity;
