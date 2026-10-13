//! Entity allocation and lifetime tracking for the initial single-owner ECS.

use super::storage::{ComponentStorage, ErasedStorage, StorageError};
use super::{Component, Entity};
use crate::runtime::{ScheduleError, Scheduler};
use core::any::TypeId;
use core::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldError {
    /// The supplied handle is stale or does not name a live entity in this world.
    EntityNotAlive,
    /// All entity indices representable by `Entity` have been exhausted.
    EntityIndexExhausted,
    /// A component storage could not reserve memory for an insertion.
    AllocationFailed,
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotAlive => f.write_str("entity is not alive in this world"),
            Self::EntityIndexExhausted => f.write_str("entity index space is exhausted"),
            Self::AllocationFailed => f.write_str("failed to reserve component storage"),
        }
    }
}

impl std::error::Error for WorldError {}

#[derive(Debug)]
struct EntitySlot {
    generation: u32,
    alive: bool,
    retired: bool,
}

/// Owns entity identity, lifetime, and component storages for one ECS world.
///
/// Each
/// component access validates the full world-local handle, and despawning an
/// entity removes its rows from every registered storage. A slot is retired if
/// its generation cannot advance without wrapping.
#[derive(Debug, Default)]
pub struct World {
    slots: Vec<EntitySlot>,
    free_indices: Vec<u32>,
    alive_count: usize,
    storages: HashMap<TypeId, Box<dyn ErasedStorage>>,
}

impl World {
    /// Creates an empty world.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_indices: Vec::new(),
            alive_count: 0,
            storages: HashMap::new(),
        }
    }

    /// Creates a new entity and returns its world-local handle.
    pub fn spawn(&mut self) -> Result<Entity, WorldError> {
        if let Some(index) = self.free_indices.pop() {
            let slot = &mut self.slots[index as usize];
            debug_assert!(!slot.alive && !slot.retired);
            slot.alive = true;
            self.alive_count += 1;
            return Ok(Entity::from_parts(index, slot.generation));
        }

        let index =
            u32::try_from(self.slots.len()).map_err(|_| WorldError::EntityIndexExhausted)?;
        self.slots.push(EntitySlot {
            generation: 0,
            alive: true,
            retired: false,
        });
        self.alive_count += 1;
        Ok(Entity::from_parts(index, 0))
    }

    /// Destroys a live entity. Stale or already-destroyed handles are rejected.
    pub fn despawn(&mut self, entity: Entity) -> Result<(), WorldError> {
        let Some(slot) = self.slots.get_mut(entity.index() as usize) else {
            return Err(WorldError::EntityNotAlive);
        };
        if !slot.alive || slot.generation != entity.generation() {
            return Err(WorldError::EntityNotAlive);
        }

        for storage in self.storages.values_mut() {
            storage.remove_entity(entity);
        }
        slot.alive = false;
        self.alive_count -= 1;
        if let Some(next_generation) = slot.generation.checked_add(1) {
            slot.generation = next_generation;
            self.free_indices.push(entity.index());
        } else {
            // Never wrap a generation: a stale handle must not become valid again.
            slot.retired = true;
        }
        Ok(())
    }

    /// Inserts a component for a live entity, returning a replaced value if any.
    pub fn insert<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<Option<T>, WorldError> {
        if !self.contains(entity) {
            return Err(WorldError::EntityNotAlive);
        }
        let storage = self
            .storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()));
        let typed = storage
            .as_any_mut()
            .downcast_mut::<ComponentStorage<T>>()
            .expect("TypeId and component storage type must agree");
        typed
            .insert(entity, component)
            .map_err(|error| match error {
                StorageError::AllocationFailed => WorldError::AllocationFailed,
            })
    }

    /// Returns a shared reference to a component attached to a live entity.
    #[must_use]
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.contains(entity) {
            return None;
        }
        self.storages
            .get(&TypeId::of::<T>())?
            .as_ref()
            .as_any()
            .downcast_ref::<ComponentStorage<T>>()?
            .get(entity)
    }

    /// Returns an exclusive reference to a component attached to a live entity.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.contains(entity) {
            return None;
        }
        self.storages
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<ComponentStorage<T>>()?
            .get_mut(entity)
    }

    /// Removes a component from a live entity and returns its value.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if !self.contains(entity) {
            return None;
        }
        self.storages
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<ComponentStorage<T>>()?
            .remove(entity)
    }

    /// Returns whether a live entity has a component of the requested type.
    #[must_use]
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        self.get::<T>(entity).is_some()
    }

    /// Returns whether the handle currently names a live entity in this world.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.slots
            .get(entity.index() as usize)
            .is_some_and(|slot| slot.alive && slot.generation == entity.generation())
    }

    /// Returns the number of currently live entities.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.alive_count
    }

    /// Returns whether this world currently has no live entities.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.alive_count == 0
    }

    /// Executes one explicit system closure against this world through Runtime.
    ///
    /// Runtime v0.1 dispatches `Scheduler::schedule` synchronously on the caller
    /// thread. The nested result keeps scheduler rejection separate from the
    /// system's own error. No background execution or parallel ECS access is
    /// implied by this adapter.
    pub fn run_system<T, E, F>(
        &mut self,
        scheduler: &Scheduler,
        system: F,
    ) -> Result<Result<T, E>, ScheduleError>
    where
        F: FnOnce(&mut World) -> Result<T, E>,
    {
        scheduler.schedule(|| system(self))
    }
}

#[cfg(test)]
mod tests {
    use super::{World, WorldError};
    use crate::runtime::Scheduler;

    fn assert_world_consistent(world: &World) {
        assert_eq!(
            world.alive_count,
            world.slots.iter().filter(|slot| slot.alive).count()
        );
        let mut seen = std::collections::HashSet::new();
        for index in &world.free_indices {
            assert!(seen.insert(*index), "duplicate free index: {index}");
            let slot = &world.slots[*index as usize];
            assert!(!slot.alive);
            assert!(!slot.retired);
        }
        for (index, slot) in world.slots.iter().enumerate() {
            if slot.alive || slot.retired {
                assert!(!seen.contains(&(index as u32)));
            } else {
                assert!(seen.contains(&(index as u32)));
            }
        }
    }

    #[test]
    fn spawn_and_despawn_track_liveness_and_counts() {
        let mut world = World::new();
        assert!(world.is_empty());
        let entity = world.spawn().unwrap();
        assert!(world.contains(entity));
        assert_eq!(world.len(), 1);
        assert!(!world.is_empty());
        world.despawn(entity).unwrap();
        assert!(!world.contains(entity));
        assert!(world.is_empty());
        assert_eq!(world.despawn(entity), Err(WorldError::EntityNotAlive));
    }

    #[test]
    fn reused_indices_advance_generation_and_reject_stale_handles() {
        let mut world = World::new();
        let stale = world.spawn().unwrap();
        world.despawn(stale).unwrap();
        let current = world.spawn().unwrap();
        assert_eq!(current.index(), stale.index());
        assert_eq!(current.generation(), stale.generation() + 1);
        assert!(!world.contains(stale));
        assert!(world.contains(current));
        assert_eq!(world.despawn(stale), Err(WorldError::EntityNotAlive));
    }

    #[test]
    fn generation_overflow_retires_slot_instead_of_wrapping() {
        let mut world = World::new();
        let entity = world.spawn().unwrap();
        let slot = &mut world.slots[entity.index() as usize];
        slot.generation = u32::MAX;
        let max_generation = super::Entity::from_parts(entity.index(), u32::MAX);
        world.despawn(max_generation).unwrap();
        let next = world.spawn().unwrap();
        assert_ne!(next.index(), max_generation.index());
        assert!(!world.contains(max_generation));
    }

    #[test]
    fn components_insert_replace_mutate_remove_and_cleanup_on_despawn() {
        let mut world = World::new();
        let entity = world.spawn().unwrap();
        assert_eq!(world.insert(entity, 10_u32), Ok(None));
        assert_eq!(world.get::<u32>(entity), Some(&10));
        assert!(world.has::<u32>(entity));
        assert_eq!(world.insert(entity, 20_u32), Ok(Some(10)));
        *world.get_mut::<u32>(entity).unwrap() += 2;
        assert_eq!(world.get::<u32>(entity), Some(&22));
        assert_eq!(world.remove::<u32>(entity), Some(22));
        assert!(!world.has::<u32>(entity));
        world.insert(entity, String::from("cleanup")).unwrap();
        world.despawn(entity).unwrap();
        assert_eq!(world.get::<String>(entity), None);
    }

    #[test]
    fn component_access_rejects_dead_and_stale_entities() {
        let mut world = World::new();
        let stale = world.spawn().unwrap();
        world.insert(stale, 7_u32).unwrap();
        world.despawn(stale).unwrap();
        assert_eq!(world.get::<u32>(stale), None);
        assert_eq!(world.insert(stale, 9_u32), Err(WorldError::EntityNotAlive));
        let current = world.spawn().unwrap();
        assert_eq!(current.index(), stale.index());
        assert_eq!(world.get::<u32>(current), None);
        assert!(!world.has::<u32>(current));
    }

    #[test]
    fn runtime_system_can_mutate_world_synchronously() {
        let scheduler = Scheduler::new();
        let mut world = World::new();
        let result = world
            .run_system(&scheduler, |world| {
                let entity = world.spawn().map_err(|_| "spawn failed")?;
                Ok::<_, &'static str>(entity)
            })
            .unwrap();
        let entity = result.unwrap();
        assert!(world.contains(entity));
        assert_eq!(world.len(), 1);
    }

    #[test]
    fn runtime_system_error_is_preserved_separately() {
        let scheduler = Scheduler::new();
        let mut world = World::new();
        let result = world.run_system(&scheduler, |_| Err::<(), _>("system failed"));
        assert_eq!(result, Ok(Err("system failed")));
        assert!(world.is_empty());
    }

    #[test]
    fn entity_slot_and_free_list_invariants_hold_across_repeated_reuse() {
        let mut world = World::new();
        let mut live = Vec::new();
        assert_world_consistent(&world);

        for step in 0..512_u32 {
            let entity = world.spawn().unwrap();
            assert!(world.contains(entity));
            live.push(entity);
            assert_world_consistent(&world);

            if step % 3 == 2 {
                let entity = live.remove(0);
                world.despawn(entity).unwrap();
                assert!(!world.contains(entity));
                assert_world_consistent(&world);
            }
        }
        while let Some(entity) = live.pop() {
            world.despawn(entity).unwrap();
            assert_world_consistent(&world);
        }
        assert!(world.is_empty());
        assert_eq!(world.len(), 0);
    }
}
