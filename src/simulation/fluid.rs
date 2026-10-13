//! Educational scalar diffusion on a bounded 2D grid (not a CFD solver).
use core::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidError {
    ZeroDimensions,
    DimensionOverflow,
    WrongCellCount,
    InvalidDiffusion,
}
impl fmt::Display for FluidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroDimensions => "grid dimensions must be non-zero",
            Self::DimensionOverflow => "grid dimensions overflow address space",
            Self::WrongCellCount => "initial field size does not match grid dimensions",
            Self::InvalidDiffusion => "diffusion coefficient must be finite and in [0, 0.25]",
        })
    }
}
impl std::error::Error for FluidError {}
/// Scalar field whose edges use a no-flux (replicated center-cell) boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct ScalarFluid {
    width: usize,
    height: usize,
    diffusion: f64,
    values: Vec<f64>,
}
impl ScalarFluid {
    pub fn new(width: usize, height: usize, diffusion: f64) -> Result<Self, FluidError> {
        if width == 0 || height == 0 {
            return Err(FluidError::ZeroDimensions);
        }
        if !diffusion.is_finite() || !(0.0..=0.25).contains(&diffusion) {
            return Err(FluidError::InvalidDiffusion);
        }
        let n = width
            .checked_mul(height)
            .ok_or(FluidError::DimensionOverflow)?;
        Ok(Self {
            width,
            height,
            diffusion,
            values: vec![0.0; n],
        })
    }
    pub fn from_values(
        width: usize,
        height: usize,
        diffusion: f64,
        values: Vec<f64>,
    ) -> Result<Self, FluidError> {
        let mut f = Self::new(width, height, diffusion)?;
        if values.len() != f.values.len() {
            return Err(FluidError::WrongCellCount);
        }
        if values.iter().any(|v| !v.is_finite()) {
            return Err(FluidError::InvalidDiffusion);
        }
        f.values = values;
        Ok(f)
    }
    pub const fn width(&self) -> usize {
        self.width
    }
    pub const fn height(&self) -> usize {
        self.height
    }
    pub const fn diffusion(&self) -> f64 {
        self.diffusion
    }
    pub fn values(&self) -> &[f64] {
        &self.values
    }
    pub fn get(&self, x: usize, y: usize) -> Option<f64> {
        if x < self.width && y < self.height {
            Some(self.values[y * self.width + x])
        } else {
            None
        }
    }
    pub fn set(&mut self, x: usize, y: usize, value: f64) -> bool {
        if x >= self.width || y >= self.height || !value.is_finite() {
            return false;
        }
        self.values[y * self.width + x] = value;
        true
    }
    /// One explicit four-neighbor diffusion step with a double buffer.
    /// The coefficient cap provides the standard 2D explicit diffusion stability bound.
    pub fn step(&mut self) {
        let old = &self.values;
        let mut next = vec![0.0; old.len()];
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width + x;
                let c = old[i];
                let l = if x > 0 { old[i - 1] } else { c };
                let r = if x + 1 < self.width { old[i + 1] } else { c };
                let u = if y > 0 { old[i - self.width] } else { c };
                let d = if y + 1 < self.height {
                    old[i + self.width]
                } else {
                    c
                };
                next[i] = c + self.diffusion * (l + r + u + d - 4.0 * c);
            }
        }
        self.values = next;
    }
    pub fn total(&self) -> f64 {
        self.values.iter().sum()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diffuses_peak_and_stays_finite() {
        let mut f = ScalarFluid::new(3, 3, 0.2).unwrap();
        f.set(1, 1, 1.0);
        f.step();
        assert!(f.get(1, 1).unwrap() < 1.0);
        assert!(f.values().iter().all(|v| v.is_finite()));
    }
    #[test]
    fn no_flux_boundary_conserves_total_with_tolerance() {
        let mut f =
            ScalarFluid::from_values(3, 2, 0.2, vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0]).unwrap();
        let before = f.total();
        f.step();
        assert!((f.total() - before).abs() < 1e-12)
    }
    #[test]
    fn rejects_unstable_diffusion() {
        assert_eq!(
            ScalarFluid::new(2, 2, 0.3).unwrap_err(),
            FluidError::InvalidDiffusion
        )
    }
}
