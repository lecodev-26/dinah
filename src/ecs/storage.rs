//! Contiguous component columns with a sparse entity-index lookup.

use super::{Component, Entity};
use core::any::Any;
use core::fmt;

/// Failure to grow a component storage without committing a partial row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StorageError {
    /// The allocator could not reserve the requested storage.
    AllocationFailed,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationFailed => f.write_str("failed to reserve component storage"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Type-erased operations needed by `World` to own and clean up storages.
pub(super) trait ErasedStorage: Any {
    fn remove_entity(&mut self, entity: Entity);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Stores one component value per entity in contiguous, index-aligned vectors.
///
/// The sparse index locates a candidate row; the complete `Entity` (including
/// its generation) is checked before returning or removing a value. `World`
/// owns these storages behind a type-erased registry.
#[derive(Debug)]
pub(super) struct ComponentStorage<T: Component> {
    entities: Vec<Entity>,
    components: Vec<T>,
    sparse: Vec<Option<usize>>,
}

impl<T: Component> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self {
            entities: Vec::new(),
            components: Vec::new(),
            sparse: Vec::new(),
        }
    }
}

impl<T: Component> ErasedStorage for ComponentStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn remove_entity(&mut self, entity: Entity) {
        let _ = self.remove(entity);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T: Component> ComponentStorage<T> {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces a component. Reusing an entity index with a new
    /// generation removes the old row before installing the new lifetime.
    pub(super) fn insert(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<Option<T>, StorageError> {
        let sparse_index = entity.index() as usize;
        let required_sparse_len = sparse_index
            .checked_add(1)
            .ok_or(StorageError::AllocationFailed)?;

        // Reserve every potentially growing vector before changing logical
        // contents, so allocation errors cannot leave columns misaligned.
        self.entities
            .try_reserve(1)
            .map_err(|_| StorageError::AllocationFailed)?;
        self.components
            .try_reserve(1)
            .map_err(|_| StorageError::AllocationFailed)?;
        if required_sparse_len > self.sparse.len() {
            self.sparse
                .try_reserve(required_sparse_len - self.sparse.len())
                .map_err(|_| StorageError::AllocationFailed)?;
            self.sparse.resize_with(required_sparse_len, || None);
        }

        if let Some(row) = self.sparse[sparse_index] {
            if self.entities[row] == entity {
                return Ok(Some(core::mem::replace(
                    &mut self.components[row],
                    component,
                )));
            }

            // Same index, different generation: the row belongs to an older
            // lifetime. Remove it only after all fallible reservations passed.
            self.remove_row(row);
        }

        let row = self.components.len();
        self.entities.push(entity);
        self.components.push(component);
        self.sparse[sparse_index] = Some(row);
        Ok(None)
    }

    pub(super) fn get(&self, entity: Entity) -> Option<&T> {
        let row = self
            .sparse
            .get(entity.index() as usize)
            .copied()
            .flatten()?;
        (self.entities[row] == entity).then(|| &self.components[row])
    }

    pub(super) fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let row = self
            .sparse
            .get(entity.index() as usize)
            .copied()
            .flatten()?;
        if self.entities[row] == entity {
            Some(&mut self.components[row])
        } else {
            None
        }
    }

    pub(super) fn remove(&mut self, entity: Entity) -> Option<T> {
        let sparse_index = entity.index() as usize;
        let row = self.sparse.get(sparse_index).copied().flatten()?;
        if self.entities[row] != entity {
            return None;
        }
        Some(self.remove_row(row))
    }

    fn remove_row(&mut self, row: usize) -> T {
        let removed_entity = self.entities.swap_remove(row);
        let removed_component = self.components.swap_remove(row);
        self.sparse[removed_entity.index() as usize] = None;

        if row < self.entities.len() {
            let moved_entity = self.entities[row];
            self.sparse[moved_entity.index() as usize] = Some(row);
        }
        removed_component
    }

    pub(super) fn len(&self) -> usize {
        debug_assert_eq!(self.entities.len(), self.components.len());
        self.components.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub(super) fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub(super) fn components(&self) -> &[T] {
        &self.components
    }

    /// Iterates over entity/component pairs in contiguous storage order.
    ///
    /// Structural changes may reorder rows; callers must not rely on a stable
    /// order across insertions or removals.
    pub(super) fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.entities.iter().copied().zip(self.components.iter())
    }

    /// Mutates each component through the shared sequential Compute primitive,
    /// retaining the entity handle aligned with its contiguous component row.
    ///
    /// This deliberately exposes no structural storage operations to the
    /// callback. It is sequential and makes no claim about parallel execution.
    pub(super) fn for_each_mut<F>(&mut self, mut operation: F)
    where
        F: FnMut(Entity, &mut T),
    {
        let entities = &self.entities;
        let mut row = 0;
        crate::compute::for_each(&mut self.components, |component| {
            let entity = entities[row];
            row += 1;
            operation(entity, component);
        });
        debug_assert_eq!(row, entities.len());
    }

    /// Mutably iterates over distinct entity/component rows without exposing
    /// structural mutation while any yielded borrow is alive.
    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.entities
            .iter()
            .copied()
            .zip(self.components.iter_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentStorage;
    use crate::ecs::Entity;

    fn entity(index: u32, generation: u32) -> Entity {
        Entity::from_parts(index, generation)
    }

    fn assert_storage_consistent<T: super::Component>(storage: &ComponentStorage<T>) {
        assert_eq!(storage.entities.len(), storage.components.len());
        assert_eq!(storage.len(), storage.entities.len());

        for (row, entity) in storage.entities.iter().copied().enumerate() {
            assert_eq!(
                storage
                    .sparse
                    .get(entity.index() as usize)
                    .copied()
                    .flatten(),
                Some(row),
                "entity {:?} at row {row} must have a sparse entry",
                entity
            );
            assert!(storage.get(entity).is_some());
        }

        for (index, row) in storage.sparse.iter().copied().enumerate() {
            if let Some(row) = row {
                assert!(row < storage.entities.len());
                assert_eq!(storage.entities[row].index() as usize, index);
                assert_eq!(
                    storage.sparse[storage.entities[row].index() as usize],
                    Some(row)
                );
            }
        }
    }

    #[test]
    fn rows_are_contiguous_and_keep_entity_component_alignment() {
        let mut storage = ComponentStorage::new();
        assert!(storage.is_empty());
        assert_eq!(storage.insert(entity(3, 0), "three"), Ok(None));
        assert_eq!(storage.insert(entity(8, 1), "eight"), Ok(None));
        assert_eq!(storage.entities(), &[entity(3, 0), entity(8, 1)]);
        assert_eq!(storage.components(), &["three", "eight"]);
        assert_eq!(storage.len(), 2);
        assert_eq!(storage.get(entity(8, 1)), Some(&"eight"));
    }

    #[test]
    fn insert_replaces_value_without_adding_a_row() {
        let mut storage = ComponentStorage::new();
        assert_eq!(storage.insert(entity(2, 0), 10), Ok(None));
        assert_eq!(storage.insert(entity(2, 0), 20), Ok(Some(10)));
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get(entity(2, 0)), Some(&20));
    }

    #[test]
    fn swap_remove_updates_moved_entity_sparse_index() {
        let mut storage = ComponentStorage::new();
        storage.insert(entity(1, 0), 10).unwrap();
        storage.insert(entity(4, 0), 40).unwrap();
        storage.insert(entity(7, 0), 70).unwrap();
        assert_eq!(storage.remove(entity(4, 0)), Some(40));
        assert_eq!(storage.len(), 2);
        assert_eq!(storage.get(entity(1, 0)), Some(&10));
        assert_eq!(storage.get(entity(7, 0)), Some(&70));
        assert_eq!(storage.entities(), &[entity(1, 0), entity(7, 0)]);
        assert_eq!(storage.components(), &[10, 70]);
    }

    #[test]
    fn stale_generation_cannot_read_or_remove_reused_index() {
        let mut storage = ComponentStorage::new();
        let stale = entity(5, 0);
        let current = entity(5, 1);
        storage.insert(stale, 11).unwrap();
        assert_eq!(storage.insert(current, 22), Ok(None));
        assert_eq!(storage.get(stale), None);
        assert_eq!(storage.remove(stale), None);
        assert_eq!(storage.get(current), Some(&22));
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn iteration_yields_aligned_entity_component_pairs_in_storage_order() {
        let mut storage = ComponentStorage::new();
        storage.insert(entity(6, 2), "six").unwrap();
        storage.insert(entity(1, 4), "one").unwrap();

        let rows: Vec<_> = storage.iter().map(|(e, value)| (e, *value)).collect();
        assert_eq!(rows, vec![(entity(6, 2), "six"), (entity(1, 4), "one")]);
        assert_eq!(ComponentStorage::<u32>::new().iter().count(), 0);
    }

    #[test]
    fn compute_for_each_mut_preserves_entity_alignment_and_row_shape() {
        let mut storage = ComponentStorage::new();
        storage.insert(entity(6, 2), 10_u32).unwrap();
        storage.insert(entity(1, 4), 20_u32).unwrap();

        let mut visited = Vec::new();
        storage.for_each_mut(|entity, value| {
            visited.push(entity);
            *value += entity.index();
        });

        assert_eq!(visited, vec![entity(6, 2), entity(1, 4)]);
        assert_eq!(storage.entities(), &[entity(6, 2), entity(1, 4)]);
        assert_eq!(storage.components(), &[16, 21]);
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn compute_for_each_mut_accepts_empty_storage() {
        let mut storage = ComponentStorage::<u32>::new();
        storage.for_each_mut(|_, value| *value += 1);
        assert!(storage.is_empty());
    }

    #[test]
    fn mutable_iteration_updates_values_without_changing_row_shape() {
        let mut storage = ComponentStorage::new();
        storage.insert(entity(0, 0), 5).unwrap();
        storage.insert(entity(3, 1), 7).unwrap();
        for (entity, value) in storage.iter_mut() {
            *value += entity.index();
        }
        assert_eq!(storage.components(), &[5, 10]);
        assert_eq!(storage.len(), 2);
    }

    #[test]
    fn mutable_access_changes_value_without_changing_row_shape() {
        let mut storage = ComponentStorage::new();
        storage.insert(entity(0, 0), 5).unwrap();
        *storage.get_mut(entity(0, 0)).unwrap() = 9;
        assert_eq!(storage.components(), &[9]);
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn sparse_and_column_invariants_hold_across_mixed_mutations() {
        let mut storage = ComponentStorage::new();
        assert_storage_consistent(&storage);

        for index in 0..64 {
            storage.insert(entity(index, 0), index * 10).unwrap();
            assert_storage_consistent(&storage);
        }
        for index in (0..64).step_by(3) {
            assert_eq!(
                storage.insert(entity(index, 0), index * 100),
                Ok(Some(index * 10))
            );
            assert_storage_consistent(&storage);
        }
        for index in (1..64).step_by(2) {
            let expected = if index % 3 == 0 {
                index * 100
            } else {
                index * 10
            };
            assert_eq!(storage.remove(entity(index, 0)), Some(expected));
            assert_storage_consistent(&storage);
        }
        for index in (0..64).step_by(4) {
            storage.insert(entity(index, 1), index + 7).unwrap();
            assert_storage_consistent(&storage);
        }
        for index in 0..64 {
            assert_eq!(
                storage.get(entity(index, 0)).is_some(),
                index % 2 == 0 && index % 4 != 0
            );
        }
        assert_storage_consistent(&storage);
    }
}
