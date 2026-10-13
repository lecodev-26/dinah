//! Explicit fixed-step simulation time primitives.
//!
//! This module deliberately does not run updates, read a wall clock, or spawn
//! workers. It only validates and advances logical simulation time.

use crate::ecs::{Entity, World};
use core::fmt;
use std::time::Duration;

/// Errors produced while constructing or advancing a [`SimulationClock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationTimeError {
    /// A fixed timestep must be greater than zero.
    ZeroDelta,
    /// Advancing would overflow the logical tick counter.
    TickOverflow,
    /// Advancing would overflow the accumulated simulation duration.
    ElapsedOverflow,
}

impl fmt::Display for SimulationTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDelta => f.write_str("fixed simulation delta must be non-zero"),
            Self::TickOverflow => f.write_str("simulation tick counter overflow"),
            Self::ElapsedOverflow => f.write_str("simulation elapsed duration overflow"),
        }
    }
}

impl std::error::Error for SimulationTimeError {}

/// Time information associated with one successfully advanced simulation step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationTime {
    /// One-based index of the step that has just completed logically.
    pub tick: u64,
    /// Total logical simulation time after this step.
    pub elapsed: Duration,
    /// Fixed duration of this step.
    pub delta: Duration,
}

/// A deterministic logical clock that advances by a validated fixed duration.
///
/// The clock has no wall-clock dependency. A failed advance leaves both the
/// tick and elapsed duration unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationClock {
    delta: Duration,
    tick: u64,
    elapsed: Duration,
}

impl SimulationClock {
    /// Creates a clock at tick zero with no elapsed simulation time.
    pub fn new(delta: Duration) -> Result<Self, SimulationTimeError> {
        if delta.is_zero() {
            return Err(SimulationTimeError::ZeroDelta);
        }

        Ok(Self {
            delta,
            tick: 0,
            elapsed: Duration::ZERO,
        })
    }

    /// Returns the configured fixed duration per step.
    pub const fn delta(&self) -> Duration {
        self.delta
    }

    /// Returns the number of successfully advanced steps.
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Returns accumulated logical simulation time.
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Advances one fixed step and returns its resulting time information.
    ///
    /// All arithmetic is checked before mutation, so overflow is atomic and
    /// does not partially advance the clock.
    pub fn advance(&mut self) -> Result<SimulationTime, SimulationTimeError> {
        let next_tick = self
            .tick
            .checked_add(1)
            .ok_or(SimulationTimeError::TickOverflow)?;
        let next_elapsed = self
            .elapsed
            .checked_add(self.delta)
            .ok_or(SimulationTimeError::ElapsedOverflow)?;

        self.tick = next_tick;
        self.elapsed = next_elapsed;

        Ok(SimulationTime {
            tick: next_tick,
            elapsed: next_elapsed,
            delta: self.delta,
        })
    }
}

/// Error returned by a [`Simulation`] step.
#[derive(Debug, PartialEq, Eq)]
pub enum SimulationStepError<E> {
    /// Logical time could not advance; the update callback was not invoked.
    Time(SimulationTimeError),
    /// The update callback failed; the logical clock was not committed.
    Update(E),
}

impl<E: fmt::Display> fmt::Display for SimulationStepError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Time(error) => write!(f, "simulation time error: {error}"),
            Self::Update(error) => write!(f, "simulation update failed: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for SimulationStepError<E> {}

/// A state value advanced through explicit, deterministic fixed-time steps.
///
/// Each call to [`Simulation::step`] invokes exactly one callback synchronously.
/// The logical clock commits only when the callback succeeds. Domain-state
/// mutations performed by a callback before it returns an error are not rolled
/// back; callers that require state atomicity must provide it themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simulation<S> {
    state: S,
    clock: SimulationClock,
}

impl<S> Simulation<S> {
    /// Creates a simulation with the supplied state and non-zero fixed delta.
    pub fn new(state: S, delta: Duration) -> Result<Self, SimulationTimeError> {
        Ok(Self {
            state,
            clock: SimulationClock::new(delta)?,
        })
    }

    /// Returns the current state by shared reference.
    pub const fn state(&self) -> &S {
        &self.state
    }

    /// Returns the current state by mutable reference.
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    /// Returns the logical clock.
    pub const fn clock(&self) -> &SimulationClock {
        &self.clock
    }

    /// Executes one synchronous update and commits logical time on success.
    ///
    /// If time would overflow, the callback is not called. If the callback
    /// returns an error, the clock remains unchanged, although mutations made
    /// to `state` by the callback are intentionally not rolled back.
    pub fn step<E>(
        &mut self,
        update: impl FnOnce(&mut S, SimulationTime) -> Result<(), E>,
    ) -> Result<SimulationTime, SimulationStepError<E>> {
        let mut candidate_clock = self.clock;
        let time = candidate_clock
            .advance()
            .map_err(SimulationStepError::Time)?;

        update(&mut self.state, time).map_err(SimulationStepError::Update)?;
        self.clock = candidate_clock;
        Ok(time)
    }
}

/// Stable, simulation-local identity for one simulated entity.
///
/// IDs are allocated monotonically and are never reused by a
/// [`SimulationEntities`] collection. They are not ECS [`crate::ecs::Entity`]
/// handles; the explicit bridge to ECS is a separate integration step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SimulationEntityId(u64);

impl SimulationEntityId {
    /// Returns the stable numeric identity assigned at insertion.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Errors returned when adding a simulated entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationEntitiesError {
    /// No further unique entity ID can be allocated without wrapping.
    IdExhausted,
}

impl fmt::Display for SimulationEntitiesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdExhausted => f.write_str("simulation entity ID space is exhausted"),
        }
    }
}

impl std::error::Error for SimulationEntitiesError {}

/// Deterministically ordered collection of simulation-owned entity state.
///
/// Iteration follows insertion order. Removing an entity preserves the relative
/// order of all remaining entities; IDs are monotonic and never reused. This
/// intentionally does not duplicate ECS component storage or entity lifetime
/// management, which are connected in a later integration step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationEntities<T> {
    next_id: Option<u64>,
    entities: Vec<(SimulationEntityId, T)>,
}

impl<T> Default for SimulationEntities<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SimulationEntities<T> {
    /// Creates an empty collection.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_id: Some(0),
            entities: Vec::new(),
        }
    }

    /// Inserts state and returns a unique ID, failing before mutation on exhaustion.
    pub fn insert(&mut self, state: T) -> Result<SimulationEntityId, SimulationEntitiesError> {
        let current_id = self.next_id.ok_or(SimulationEntitiesError::IdExhausted)?;
        let id = SimulationEntityId(current_id);
        self.entities.push((id, state));
        self.next_id = current_id.checked_add(1);
        Ok(id)
    }

    /// Returns state for an ID currently in this collection.
    #[must_use]
    pub fn get(&self, id: SimulationEntityId) -> Option<&T> {
        self.entities
            .iter()
            .find_map(|(candidate, state)| (*candidate == id).then_some(state))
    }

    /// Returns mutable state for an ID currently in this collection.
    pub fn get_mut(&mut self, id: SimulationEntityId) -> Option<&mut T> {
        self.entities
            .iter_mut()
            .find_map(|(candidate, state)| (*candidate == id).then_some(state))
    }

    /// Removes state and returns it without reordering remaining entities.
    pub fn remove(&mut self, id: SimulationEntityId) -> Option<T> {
        let index = self
            .entities
            .iter()
            .position(|(candidate, _)| *candidate == id)?;
        Some(self.entities.remove(index).1)
    }

    /// Iterates over `(ID, state)` pairs in deterministic insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (SimulationEntityId, &T)> {
        self.entities.iter().map(|(id, state)| (*id, state))
    }

    /// Returns the number of currently stored entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns whether the collection has no entities.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// Errors produced when binding simulation identities to ECS entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationEntityBindingError {
    /// The ECS handle is stale or is not alive in the supplied world.
    EntityNotAlive,
    /// This live ECS entity already has a simulation identity in this binding set.
    EntityAlreadyBound,
    /// No further simulation identity can be allocated without wrapping.
    IdExhausted,
}

impl fmt::Display for SimulationEntityBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotAlive => f.write_str("ECS entity is not alive in the supplied world"),
            Self::EntityAlreadyBound => f.write_str("ECS entity already has a simulation identity"),
            Self::IdExhausted => f.write_str("simulation entity ID space is exhausted"),
        }
    }
}

impl std::error::Error for SimulationEntityBindingError {}

/// Explicit, world-validated bridge between simulation IDs and ECS handles.
///
/// This type does not own either the [`World`] or entity lifetimes. Callers
/// supply the owning world when binding and resolving. Resolution revalidates
/// liveness, so a despawned entity or a stale generation never resolves even if
/// its old binding has not yet been removed. Each live ECS handle can be bound
/// at most once per bridge; IDs are monotonic and are never reused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationEntityBindings {
    next_id: Option<u64>,
    bindings: Vec<(SimulationEntityId, Entity)>,
}

impl Default for SimulationEntityBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationEntityBindings {
    /// Creates an empty simulation-to-ECS binding set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_id: Some(0),
            bindings: Vec::new(),
        }
    }

    /// Binds a live ECS entity to a new stable simulation identity.
    pub fn bind(
        &mut self,
        world: &World,
        entity: Entity,
    ) -> Result<SimulationEntityId, SimulationEntityBindingError> {
        if !world.contains(entity) {
            return Err(SimulationEntityBindingError::EntityNotAlive);
        }
        if self.bindings.iter().any(|(_, bound)| *bound == entity) {
            return Err(SimulationEntityBindingError::EntityAlreadyBound);
        }
        let value = self
            .next_id
            .ok_or(SimulationEntityBindingError::IdExhausted)?;
        let id = SimulationEntityId(value);
        self.bindings.push((id, entity));
        self.next_id = value.checked_add(1);
        Ok(id)
    }

    /// Resolves an ID only while its ECS entity is still alive in `world`.
    #[must_use]
    pub fn resolve(&self, world: &World, id: SimulationEntityId) -> Option<Entity> {
        let entity = self
            .bindings
            .iter()
            .find_map(|(candidate, entity)| (*candidate == id).then_some(*entity))?;
        world.contains(entity).then_some(entity)
    }

    /// Removes a binding without despawning the ECS entity.
    pub fn unbind(&mut self, id: SimulationEntityId) -> Option<Entity> {
        let index = self
            .bindings
            .iter()
            .position(|(candidate, _)| *candidate == id)?;
        Some(self.bindings.remove(index).1)
    }

    /// Returns the number of bindings, including any whose ECS entities were despawned.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Returns whether this binding set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Simulation, SimulationClock, SimulationEntities, SimulationEntitiesError,
        SimulationEntityBindingError, SimulationEntityBindings, SimulationStepError,
        SimulationTimeError,
    };
    use std::time::Duration;

    use crate::ecs::World;

    #[test]
    fn simulation_entity_bindings_validate_liveness_and_reject_duplicates() {
        let mut world = World::new();
        let entity = world.spawn().unwrap();
        let mut bindings = SimulationEntityBindings::new();
        let id = bindings.bind(&world, entity).unwrap();

        assert_eq!(id.get(), 0);
        assert_eq!(bindings.resolve(&world, id), Some(entity));
        assert_eq!(bindings.len(), 1);
        assert_eq!(
            bindings.bind(&world, entity),
            Err(SimulationEntityBindingError::EntityAlreadyBound)
        );

        world.despawn(entity).unwrap();
        assert_eq!(bindings.resolve(&world, id), None);
        let replacement = world.spawn().unwrap();
        assert_ne!(replacement, entity);
        assert_eq!(bindings.resolve(&world, id), None);
        assert_eq!(bindings.unbind(id), Some(entity));
        assert!(bindings.is_empty());
    }

    #[test]
    fn simulation_entity_bindings_reject_dead_handles_without_mutation() {
        let mut world = World::new();
        let entity = world.spawn().unwrap();
        world.despawn(entity).unwrap();
        let mut bindings = SimulationEntityBindings::new();

        assert_eq!(
            bindings.bind(&world, entity),
            Err(SimulationEntityBindingError::EntityNotAlive)
        );
        assert!(bindings.is_empty());
    }

    #[test]
    fn simulation_steps_in_order_and_commits_time_after_success() {
        let mut simulation = Simulation::new(vec![0_u64], Duration::from_millis(10)).unwrap();
        let first = simulation
            .step(|state, time| {
                assert_eq!(time.tick, 1);
                assert_eq!(time.elapsed, Duration::from_millis(10));
                state.push(time.tick);
                Ok::<_, ()>(())
            })
            .unwrap();
        let second = simulation
            .step(|state, time| {
                assert_eq!(time.tick, 2);
                state.push(time.tick);
                Ok::<_, ()>(())
            })
            .unwrap();

        assert_eq!(first.tick, 1);
        assert_eq!(second.tick, 2);
        assert_eq!(simulation.state(), &[0, 1, 2]);
        assert_eq!(simulation.clock().tick(), 2);
        assert_eq!(simulation.clock().elapsed(), Duration::from_millis(20));
    }

    #[test]
    fn failed_update_does_not_commit_clock() {
        let mut simulation = Simulation::new(0_u32, Duration::from_millis(10)).unwrap();
        let result = simulation.step(|state, _time| {
            *state += 1;
            Err::<(), _>("domain error")
        });

        assert_eq!(result, Err(SimulationStepError::Update("domain error")));
        assert_eq!(*simulation.state(), 1, "domain state is not rolled back");
        assert_eq!(simulation.clock().tick(), 0);
        assert_eq!(simulation.clock().elapsed(), Duration::ZERO);
    }

    #[test]
    fn time_overflow_prevents_update_callback() {
        let mut simulation = Simulation::new(0_u32, Duration::from_nanos(1)).unwrap();
        simulation.clock.tick = u64::MAX;
        let mut called = false;

        let result = simulation.step(|state, _time| {
            called = true;
            *state += 1;
            Ok::<_, ()>(())
        });

        assert_eq!(
            result,
            Err(SimulationStepError::Time(SimulationTimeError::TickOverflow))
        );
        assert!(!called);
        assert_eq!(*simulation.state(), 0);
        assert_eq!(simulation.clock().tick(), u64::MAX);
    }

    #[test]
    fn simulated_entities_keep_stable_ids_and_insertion_order() {
        let mut entities = SimulationEntities::new();
        assert!(entities.is_empty());

        let first = entities.insert("first").unwrap();
        let second = entities.insert("second").unwrap();
        let third = entities.insert("third").unwrap();

        assert_eq!((first.get(), second.get(), third.get()), (0, 1, 2));
        assert_eq!(
            entities.iter().collect::<Vec<_>>(),
            vec![(first, &"first"), (second, &"second"), (third, &"third")]
        );
        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn simulated_entity_removal_preserves_remaining_order_and_never_reuses_ids() {
        let mut entities = SimulationEntities::new();
        let first = entities.insert(10).unwrap();
        let second = entities.insert(20).unwrap();
        let third = entities.insert(30).unwrap();

        assert_eq!(entities.remove(second), Some(20));
        assert_eq!(entities.get(second), None);
        assert_eq!(
            entities.iter().collect::<Vec<_>>(),
            vec![(first, &10), (third, &30)]
        );
        let fourth = entities.insert(40).unwrap();
        assert_eq!(fourth.get(), 3);
        *entities.get_mut(first).unwrap() += 1;
        assert_eq!(entities.get(first), Some(&11));
        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn simulated_entity_id_exhaustion_does_not_mutate_collection() {
        let mut entities = SimulationEntities::<u8>::new();
        entities.next_id = Some(u64::MAX);

        let last_id = entities.insert(7).unwrap();
        assert_eq!(last_id.get(), u64::MAX);
        assert_eq!(
            entities.insert(8),
            Err(SimulationEntitiesError::IdExhausted)
        );
        assert_eq!(entities.len(), 1);
        assert_eq!(entities.get(last_id), Some(&7));
        assert_eq!(entities.next_id, None);
    }

    #[test]
    fn rejects_zero_delta() {
        assert_eq!(
            SimulationClock::new(Duration::ZERO),
            Err(SimulationTimeError::ZeroDelta)
        );
    }

    #[test]
    fn advances_by_fixed_delta_without_wall_clock() {
        let delta = Duration::from_millis(20);
        let mut clock = SimulationClock::new(delta).unwrap();

        assert_eq!(clock.tick(), 0);
        assert_eq!(clock.elapsed(), Duration::ZERO);

        let first = clock.advance().unwrap();
        assert_eq!(first.tick, 1);
        assert_eq!(first.delta, delta);
        assert_eq!(first.elapsed, Duration::from_millis(20));

        let second = clock.advance().unwrap();
        assert_eq!(second.tick, 2);
        assert_eq!(second.elapsed, Duration::from_millis(40));
        assert_eq!(clock.tick(), 2);
    }

    #[test]
    fn tick_overflow_does_not_mutate_clock() {
        let mut clock = SimulationClock::new(Duration::from_nanos(1)).unwrap();
        clock.tick = u64::MAX;
        let elapsed_before = clock.elapsed;

        assert_eq!(clock.advance(), Err(SimulationTimeError::TickOverflow));
        assert_eq!(clock.tick(), u64::MAX);
        assert_eq!(clock.elapsed(), elapsed_before);
    }

    #[test]
    fn elapsed_overflow_does_not_mutate_clock() {
        let mut clock = SimulationClock::new(Duration::from_secs(1)).unwrap();
        clock.elapsed = Duration::MAX;
        let tick_before = clock.tick;

        assert_eq!(clock.advance(), Err(SimulationTimeError::ElapsedOverflow));
        assert_eq!(clock.tick(), tick_before);
        assert_eq!(clock.elapsed(), Duration::MAX);
    }
}
