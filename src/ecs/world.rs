//! Entity allocation and lifetime tracking for the initial single-owner ECS.

use super::Entity;
use crate::runtime::{ScheduleError, Scheduler};
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldError {
    /// The supplied handle is stale or does not name a live entity in this world.
    EntityNotAlive,
    /// All entity indices representable by `Entity` have been exhausted.
    EntityIndexExhausted,
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotAlive => f.write_str("entity is not alive in this world"),
            Self::EntityIndexExhausted => f.write_str("entity index space is exhausted"),
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

/// Owns entity identity and lifetime for one ECS world.
///
/// Component storage is added in a later roadmap step. This initial `World`
/// allocates handles, validates their current generation, and reuses indices
/// only after advancing their generation. A slot is permanently retired if
/// its generation cannot be advanced without wrapping.
#[derive(Debug, Default)]
pub struct World {
    slots: Vec<EntitySlot>,
    free_indices: Vec<u32>,
    alive_count: usize,
}

impl World {
    /// Creates an empty world.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_indices: Vec::new(),
            alive_count: 0,
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
