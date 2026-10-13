//! Small deterministic grid-based particle physics for ASCII-style worlds.
//!
//! This is a discrete educational model, not a general rigid-body solver.
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsError {
    ZeroDimensions,
    OutOfBounds,
    DimensionOverflow,
}
impl fmt::Display for PhysicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroDimensions => "grid dimensions must be non-zero",
            Self::OutOfBounds => "coordinate is outside the grid",
            Self::DimensionOverflow => "grid dimensions overflow address space",
        })
    }
}
impl std::error::Error for PhysicsError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
}

/// Bounded grid with fixed obstacles and deterministic, one-cell movement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsciiPhysics {
    width: usize,
    height: usize,
    obstacles: Vec<bool>,
    particles: Vec<Particle>,
}
impl AsciiPhysics {
    pub fn new(width: usize, height: usize) -> Result<Self, PhysicsError> {
        if width == 0 || height == 0 {
            return Err(PhysicsError::ZeroDimensions);
        }
        let len = width
            .checked_mul(height)
            .ok_or(PhysicsError::DimensionOverflow)?;
        Ok(Self {
            width,
            height,
            obstacles: vec![false; len],
            particles: Vec::new(),
        })
    }
    pub const fn width(&self) -> usize {
        self.width
    }
    pub const fn height(&self) -> usize {
        self.height
    }
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
    pub fn set_obstacle(&mut self, x: usize, y: usize, blocked: bool) -> Result<(), PhysicsError> {
        if x >= self.width || y >= self.height {
            return Err(PhysicsError::OutOfBounds);
        }
        let i = self.index(x, y);
        self.obstacles[i] = blocked;
        Ok(())
    }
    pub fn add_particle(&mut self, particle: Particle) -> Result<usize, PhysicsError> {
        if !self.inside(particle.x, particle.y) {
            return Err(PhysicsError::OutOfBounds);
        }
        self.particles.push(particle);
        Ok(self.particles.len() - 1)
    }
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }
    fn inside(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }
    fn blocked(&self, x: i32, y: i32) -> bool {
        !self.inside(x, y) || self.obstacles[self.index(x as usize, y as usize)]
    }
    /// Advances each particle once in insertion order; blocked axes bounce by reversing velocity.
    pub fn step(&mut self) {
        for i in 0..self.particles.len() {
            let p = self.particles[i];
            let nx = p.x.saturating_add(p.vx.signum());
            let ny = p.y.saturating_add(p.vy.signum());
            let mut next = p;
            if !self.blocked(nx, p.y) {
                next.x = nx;
            } else {
                next.vx = p.vx.saturating_neg();
            }
            if !self.blocked(next.x, ny) {
                next.y = ny;
            } else {
                next.vy = p.vy.saturating_neg();
            }
            self.particles[i] = next;
        }
    }
    /// Renders obstacles as `#`, particles as `o`, and empty cells as spaces.
    pub fn render(&self) -> String {
        let mut cells: Vec<char> = self
            .obstacles
            .iter()
            .map(|b| if *b { '#' } else { ' ' })
            .collect();
        for p in &self.particles {
            if self.inside(p.x, p.y) {
                cells[self.index(p.x as usize, p.y as usize)] = 'o';
            }
        }
        let mut out = String::with_capacity(self.width.saturating_mul(self.height + 1));
        for y in 0..self.height {
            for x in 0..self.width {
                out.push(cells[self.index(x, y)]);
            }
            if y + 1 < self.height {
                out.push('\n');
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_grid_and_particle() {
        assert_eq!(
            AsciiPhysics::new(0, 2).unwrap_err(),
            PhysicsError::ZeroDimensions
        );
        let mut w = AsciiPhysics::new(2, 2).unwrap();
        assert_eq!(
            w.add_particle(Particle {
                x: 2,
                y: 0,
                vx: 1,
                vy: 0
            }),
            Err(PhysicsError::OutOfBounds)
        );
    }
    #[test]
    fn bounces_at_wall_reproducibly() {
        let mut w = AsciiPhysics::new(3, 1).unwrap();
        w.add_particle(Particle {
            x: 1,
            y: 0,
            vx: 1,
            vy: 0,
        })
        .unwrap();
        w.step();
        assert_eq!(
            w.particles()[0],
            Particle {
                x: 2,
                y: 0,
                vx: 1,
                vy: 0
            }
        );
        w.step();
        assert_eq!(
            w.particles()[0],
            Particle {
                x: 2,
                y: 0,
                vx: -1,
                vy: 0
            }
        );
    }
    #[test]
    fn obstacle_is_rendered_and_blocks_motion() {
        let mut w = AsciiPhysics::new(3, 1).unwrap();
        w.set_obstacle(1, 0, true).unwrap();
        w.add_particle(Particle {
            x: 0,
            y: 0,
            vx: 1,
            vy: 0,
        })
        .unwrap();
        w.step();
        assert_eq!(w.particles()[0].x, 0);
        assert_eq!(w.render(), "o# ");
    }
}
