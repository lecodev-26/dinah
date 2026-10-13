//! Synchronous, bounded two-dimensional cellular automata.
use core::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellularError {
    ZeroDimensions,
    DimensionOverflow,
    WrongCellCount,
}
impl fmt::Display for CellularError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroDimensions => "grid dimensions must be non-zero",
            Self::DimensionOverflow => "grid dimensions overflow address space",
            Self::WrongCellCount => "initial cell count does not match grid dimensions",
        })
    }
}
impl std::error::Error for CellularError {}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    Dead,
    Wrap,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellularGrid {
    width: usize,
    height: usize,
    cells: Vec<bool>,
    boundary: Boundary,
}
impl CellularGrid {
    pub fn new(width: usize, height: usize, boundary: Boundary) -> Result<Self, CellularError> {
        if width == 0 || height == 0 {
            return Err(CellularError::ZeroDimensions);
        }
        let n = width
            .checked_mul(height)
            .ok_or(CellularError::DimensionOverflow)?;
        Ok(Self {
            width,
            height,
            cells: vec![false; n],
            boundary,
        })
    }
    pub fn from_cells(
        width: usize,
        height: usize,
        boundary: Boundary,
        cells: Vec<bool>,
    ) -> Result<Self, CellularError> {
        let mut g = Self::new(width, height, boundary)?;
        if cells.len() != g.cells.len() {
            return Err(CellularError::WrongCellCount);
        }
        g.cells = cells;
        Ok(g)
    }
    pub const fn width(&self) -> usize {
        self.width
    }
    pub const fn height(&self) -> usize {
        self.height
    }
    pub fn cells(&self) -> &[bool] {
        &self.cells
    }
    pub fn get(&self, x: usize, y: usize) -> Option<bool> {
        if x < self.width && y < self.height {
            Some(self.cells[y * self.width + x])
        } else {
            None
        }
    }
    pub fn set(&mut self, x: usize, y: usize, alive: bool) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.cells[y * self.width + x] = alive;
        true
    }
    fn alive_at(&self, x: isize, y: isize) -> bool {
        let (x, y) = match self.boundary {
            Boundary::Dead => {
                if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
                    return false;
                }
                (x as usize, y as usize)
            }
            Boundary::Wrap => (
                x.rem_euclid(self.width as isize) as usize,
                y.rem_euclid(self.height as isize) as usize,
            ),
        };
        self.cells[y * self.width + x]
    }
    /// Applies Conway's Game of Life using a snapshot; all cells change simultaneously.
    pub fn step_life(&mut self) {
        let mut next = vec![false; self.cells.len()];
        for y in 0..self.height {
            for x in 0..self.width {
                let mut n = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx != 0 || dy != 0 {
                            n += usize::from(self.alive_at(x as isize + dx, y as isize + dy));
                        }
                    }
                }
                let i = y * self.width + x;
                next[i] = if self.cells[i] {
                    n == 2 || n == 3
                } else {
                    n == 3
                };
            }
        }
        self.cells = next;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blinker_oscillates() {
        let mut g = CellularGrid::new(5, 5, Boundary::Dead).unwrap();
        g.set(2, 1, true);
        g.set(2, 2, true);
        g.set(2, 3, true);
        g.step_life();
        assert_eq!((0..5).filter(|x| g.get(*x, 2) == Some(true)).count(), 3);
        assert_eq!((0..5).filter(|y| g.get(2, *y) == Some(true)).count(), 1);
        g.step_life();
        assert!(g.get(2, 1).unwrap() && g.get(2, 2).unwrap() && g.get(2, 3).unwrap());
    }
    #[test]
    fn dead_boundary_does_not_wrap() {
        let mut g = CellularGrid::new(2, 2, Boundary::Dead).unwrap();
        g.set(0, 0, true);
        assert!(!g.alive_at(-1, 0));
    }
    #[test]
    fn validates_initial_size() {
        assert_eq!(
            CellularGrid::from_cells(2, 2, Boundary::Dead, vec![true]),
            Err(CellularError::WrongCellCount)
        );
    }
}
