use core::fmt;

/// A lightweight handle to an entity owned by one ECS world.
///
/// An entity is identified by an index and a generation. The generation lets a
/// world distinguish a newly created entity from an older entity whose index
/// has since been reused. Handles are world-local and do not imply global or
/// cross-process uniqueness.
///
/// The fields and constructor are intentionally private: callers obtain valid
/// handles from `World` rather than manufacturing handles that may refer to
/// nonexistent or stale entities. Read-only accessors are provided for
/// diagnostics and indexing integrations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    index: u32,
    generation: u32,
}

impl Entity {
    /// Creates a handle for ECS internals.
    ///
    /// The caller must ensure that the index is allocated and that the
    /// generation matches the owning world's live-entity table.
    #[allow(dead_code)] // Used by World when entity allocation is implemented.
    pub(crate) const fn from_parts(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Returns this handle's world-local entity index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// Returns this handle's generation, used to reject stale handles.
    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.index, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::Entity;
    use core::mem::size_of;
    use std::collections::HashSet;

    #[test]
    fn entity_exposes_its_index_and_generation_without_mutation() {
        let entity = Entity::from_parts(17, 3);
        assert_eq!(entity.index(), 17);
        assert_eq!(entity.generation(), 3);
        assert_eq!(entity.to_string(), "17v3");
        assert_eq!(entity.index(), 17);
        assert_eq!(entity.generation(), 3);
    }

    #[test]
    fn entity_identity_includes_generation() {
        let first_lifetime = Entity::from_parts(4, 1);
        let reused_index = Entity::from_parts(4, 2);
        let same_handle = Entity::from_parts(4, 1);

        assert_ne!(first_lifetime, reused_index);
        assert_eq!(first_lifetime, same_handle);
        assert_ne!(
            first_lifetime.cmp(&reused_index),
            core::cmp::Ordering::Equal
        );

        let mut handles = HashSet::new();
        handles.insert(first_lifetime);
        handles.insert(reused_index);
        handles.insert(same_handle);
        assert_eq!(handles.len(), 2);
    }

    #[test]
    fn entity_handle_is_compact_and_copyable() {
        assert_eq!(size_of::<Entity>(), size_of::<u64>());
        let entity = Entity::from_parts(u32::MAX, u32::MAX);
        let copied = entity;
        assert_eq!(entity, copied);
        assert_eq!(copied.index(), u32::MAX);
        assert_eq!(copied.generation(), u32::MAX);
    }
}
