//! Dependency-free 2D ray intersection with axis-aligned rectangles.
use core::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaycastError {
    NonFinite,
    NegativeExtent,
    ZeroDirection,
}
impl fmt::Display for RaycastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "ray and rectangle values must be finite",
            Self::NegativeExtent => "rectangle extents must be non-negative",
            Self::ZeroDirection => "ray direction must be non-zero",
        })
    }
}
impl std::error::Error for RaycastError {}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray2 {
    pub origin: Vec2,
    pub direction: Vec2,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2 {
    pub min: Vec2,
    pub max: Vec2,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    pub distance: f64,
    pub point: Vec2,
}
impl Ray2 {
    pub fn new(origin: Vec2, direction: Vec2) -> Result<Self, RaycastError> {
        if ![origin.x, origin.y, direction.x, direction.y]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err(RaycastError::NonFinite);
        }
        if direction.x == 0.0 && direction.y == 0.0 {
            return Err(RaycastError::ZeroDirection);
        }
        Ok(Self { origin, direction })
    }
}
impl Aabb2 {
    pub fn new(min: Vec2, max: Vec2) -> Result<Self, RaycastError> {
        if ![min.x, min.y, max.x, max.y].iter().all(|v| v.is_finite()) {
            return Err(RaycastError::NonFinite);
        }
        if max.x < min.x || max.y < min.y {
            return Err(RaycastError::NegativeExtent);
        }
        Ok(Self { min, max })
    }
    /// Returns the nearest non-negative intersection distance; an origin inside returns distance zero.
    pub fn intersect(&self, ray: Ray2) -> Option<RayHit> {
        if self.contains(ray.origin) {
            return Some(RayHit {
                distance: 0.0,
                point: ray.origin,
            });
        }
        let mut near = f64::NEG_INFINITY;
        let mut far = f64::INFINITY;
        for (o, d, min, max) in [
            (ray.origin.x, ray.direction.x, self.min.x, self.max.x),
            (ray.origin.y, ray.direction.y, self.min.y, self.max.y),
        ] {
            if d == 0.0 {
                if o < min || o > max {
                    return None;
                }
            } else {
                let a = (min - o) / d;
                let b = (max - o) / d;
                near = near.max(a.min(b));
                far = far.min(a.max(b));
                if near > far {
                    return None;
                }
            }
        }
        let distance = if near >= 0.0 {
            near
        } else if far >= 0.0 {
            far
        } else {
            return None;
        };
        let point = Vec2 {
            x: ray.origin.x + ray.direction.x * distance,
            y: ray.origin.y + ray.direction.y * distance,
        };
        Some(RayHit { distance, point })
    }
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn box2() -> Aabb2 {
        Aabb2::new(Vec2 { x: 2.0, y: 2.0 }, Vec2 { x: 4.0, y: 4.0 }).unwrap()
    }
    #[test]
    fn hits_nearest_face() {
        let r = Ray2::new(Vec2 { x: 0.0, y: 3.0 }, Vec2 { x: 1.0, y: 0.0 }).unwrap();
        assert_eq!(
            box2().intersect(r).unwrap(),
            RayHit {
                distance: 2.0,
                point: Vec2 { x: 2.0, y: 3.0 }
            }
        )
    }
    #[test]
    fn misses_and_handles_parallel_axis() {
        let r = Ray2::new(Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 1.0, y: 0.0 }).unwrap();
        assert!(box2().intersect(r).is_none())
    }
    #[test]
    fn origin_inside_is_zero() {
        let r = Ray2::new(Vec2 { x: 3.0, y: 3.0 }, Vec2 { x: 0.0, y: 1.0 }).unwrap();
        assert_eq!(box2().intersect(r).unwrap().distance, 0.0)
    }
    #[test]
    fn rejects_zero_ray() {
        assert_eq!(
            Ray2::new(Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 0.0, y: 0.0 }),
            Err(RaycastError::ZeroDirection)
        )
    }
}
