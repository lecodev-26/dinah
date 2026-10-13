//! Small 2D flocking model with snapshot-based, order-independent updates.
use core::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmError {
    InvalidParameter,
    NonFiniteAgent,
}
impl fmt::Display for SwarmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidParameter => "swarm parameters must be finite and non-negative",
            Self::NonFiniteAgent => "agent position and velocity must be finite",
        })
    }
}
impl std::error::Error for SwarmError {}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Agent2 {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Flock2D {
    agents: Vec<Agent2>,
    neighbor_radius: f64,
    steering: f64,
    max_speed: f64,
}
impl Flock2D {
    pub fn new(
        agents: Vec<Agent2>,
        neighbor_radius: f64,
        steering: f64,
        max_speed: f64,
    ) -> Result<Self, SwarmError> {
        if !neighbor_radius.is_finite()
            || neighbor_radius < 0.0
            || !steering.is_finite()
            || steering < 0.0
            || !max_speed.is_finite()
            || max_speed < 0.0
        {
            return Err(SwarmError::InvalidParameter);
        }
        if agents
            .iter()
            .any(|a| [a.x, a.y, a.vx, a.vy].iter().any(|v| !v.is_finite()))
        {
            return Err(SwarmError::NonFiniteAgent);
        }
        Ok(Self {
            agents,
            neighbor_radius,
            steering,
            max_speed,
        })
    }
    pub fn agents(&self) -> &[Agent2] {
        &self.agents
    }
    /// Computes every next agent from the same snapshot; the input vector order cannot affect neighbor sums.
    pub fn step(&mut self) {
        let old = &self.agents;
        let mut next = Vec::with_capacity(old.len());
        let r2 = self.neighbor_radius * self.neighbor_radius;
        for (i, a) in old.iter().enumerate() {
            let (mut px, mut py, mut vx, mut vy, mut count) = (0.0, 0.0, 0.0, 0.0, 0usize);
            for (j, b) in old.iter().enumerate() {
                if i == j {
                    continue;
                }
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                if dx * dx + dy * dy <= r2 {
                    px += b.x;
                    py += b.y;
                    vx += b.vx;
                    vy += b.vy;
                    count += 1
                }
            }
            let (mut ax, mut ay) = (0.0, 0.0);
            if count > 0 {
                let n = count as f64;
                ax = (px / n - a.x) * self.steering + (vx / n - a.vx) * self.steering;
                ay = (py / n - a.y) * self.steering + (vy / n - a.vy) * self.steering;
            }
            let (mut nx, mut ny) = (a.vx + ax, a.vy + ay);
            let speed = (nx * nx + ny * ny).sqrt();
            if speed > self.max_speed && speed > 0.0 {
                let scale = self.max_speed / speed;
                nx *= scale;
                ny *= scale;
            }
            next.push(Agent2 {
                x: a.x + nx,
                y: a.y + ny,
                vx: nx,
                vy: ny,
            });
        }
        self.agents = next;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Vec<Agent2> {
        vec![
            Agent2 {
                x: 0.0,
                y: 0.0,
                vx: 1.0,
                vy: 0.0,
            },
            Agent2 {
                x: 1.0,
                y: 0.0,
                vx: 0.0,
                vy: 1.0,
            },
            Agent2 {
                x: 8.0,
                y: 8.0,
                vx: -1.0,
                vy: 0.0,
            },
        ]
    }
    #[test]
    fn replay_is_deterministic() {
        let mut a = Flock2D::new(sample(), 3.0, 0.05, 2.0).unwrap();
        let mut b = a.clone();
        for _ in 0..8 {
            a.step();
            b.step()
        }
        assert_eq!(a, b)
    }
    #[test]
    fn speed_is_bounded() {
        let mut f = Flock2D::new(sample(), 20.0, 10.0, 1.25).unwrap();
        f.step();
        assert!(f
            .agents()
            .iter()
            .all(|a| (a.vx * a.vx + a.vy * a.vy).sqrt() <= 1.2500000001))
    }
    #[test]
    fn rejects_nonfinite_agent() {
        let a = Agent2 {
            x: f64::NAN,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
        };
        assert_eq!(
            Flock2D::new(vec![a], 1.0, 0.1, 1.0).unwrap_err(),
            SwarmError::NonFiniteAgent
        )
    }
}
