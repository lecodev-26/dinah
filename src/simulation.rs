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

#[cfg(test)]
mod tests {
    use super::{SimulationClock, SimulationTimeError};
    use std::time::Duration;

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
