//! Deterministic connection liveness and reconnect-delay policy.
use core::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Liveness {
    Connected,
    TimedOut,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectError {
    ZeroTimeout,
    ZeroBackoff,
    InvalidBackoffRange,
}
impl core::fmt::Display for DisconnectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroTimeout => f.write_str("liveness timeout must be non-zero"),
            Self::ZeroBackoff => f.write_str("reconnect backoff must be non-zero"),
            Self::InvalidBackoffRange => {
                f.write_str("maximum reconnect backoff must be at least the initial delay")
            }
        }
    }
}
impl std::error::Error for DisconnectError {}

/// Tracks caller-supplied logical time; it never reads wall-clock time or reconnects itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConnectionMonitor {
    timeout: Duration,
    last_activity: Duration,
}
impl ConnectionMonitor {
    pub fn new(timeout: Duration, now: Duration) -> Result<Self, DisconnectError> {
        if timeout.is_zero() {
            return Err(DisconnectError::ZeroTimeout);
        }
        Ok(Self {
            timeout,
            last_activity: now,
        })
    }
    pub const fn last_activity(&self) -> Duration {
        self.last_activity
    }
    pub fn observe_activity(&mut self, now: Duration) {
        if now > self.last_activity {
            self.last_activity = now;
        }
    }
    pub fn status(&self, now: Duration) -> Liveness {
        if now.saturating_sub(self.last_activity) >= self.timeout {
            Liveness::TimedOut
        } else {
            Liveness::Connected
        }
    }
}

/// Exponential reconnect delay calculator. The caller owns attempts and actual reconnection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconnectBackoff {
    initial: Duration,
    maximum: Duration,
}
impl ReconnectBackoff {
    pub fn new(initial: Duration, maximum: Duration) -> Result<Self, DisconnectError> {
        if initial.is_zero() {
            return Err(DisconnectError::ZeroBackoff);
        }
        if maximum < initial {
            return Err(DisconnectError::InvalidBackoffRange);
        }
        Ok(Self { initial, maximum })
    }
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let mut delay = self.initial;
        for _ in 0..attempt.min(63) {
            delay = delay
                .checked_mul(2)
                .unwrap_or(self.maximum)
                .min(self.maximum);
            if delay == self.maximum {
                break;
            }
        }
        delay
    }
}
