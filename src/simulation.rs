//! Explicit fixed-step simulation time primitives.
//!
//! This module deliberately does not run updates, read a wall clock, or spawn
//! workers. It only validates and advances logical simulation time.

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

#[cfg(test)]
mod tests {
    use super::{Simulation, SimulationClock, SimulationStepError, SimulationTimeError};
    use std::time::Duration;

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
