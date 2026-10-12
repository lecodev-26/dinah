//! Entity-component-system foundations.
//!
//! The ECS is introduced incrementally. Entity handles are defined first;
//! component storage and world ownership are added by later roadmap steps.

mod entity;

pub use entity::Entity;
