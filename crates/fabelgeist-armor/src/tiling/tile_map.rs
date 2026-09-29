//! Where a plate's own coordinates land on its grid.
//!
//! A plate is laid on the surface by walking out from its centre: up and
//! down along the surface in short steps, then across from each point
//! reached, each step taken in the frame where it starts. Distances along
//! those walks are the plate's own, however the grid's spacing or the
//! surface's curvature changes under it, which a single step from the
//! centre would not keep. The walk is taken once, on a coarse lattice over
//! the plate, and any point of the plate is interpolated from it.

use super::surface::{GridPoint, GridSurface};

/// Lattice cells across a plate, and along it.
const CELLS: [usize; 2] = [4, 8];
/// How far past its outline the lattice reaches, as a share of the plate, so
/// that cord leaving a hole or turning under a foot lands on it.
const MARGIN: f32 = 0.15;

/// A plate's coordinates on its grid.
#[derive(Clone, Debug)]
pub(super) struct TileMap {
    /// The lattice's corner and cell size in the plate's coordinates.
    corner: [f32; 2],
    cell: [f32; 2],
    /// Nodes row by row from the bottom, `CELLS[0] + 1` across.
    nodes: Vec<GridPoint>,
}

impl TileMap {
    /// Walk a `width` by `height` plate out from `center`. `None` where the
    /// surface has no frame to step in.
    pub(super) fn walk(
        surface: &GridSurface,
        center: GridPoint,
        width: f32,
        height: f32,
    ) -> Option<Self> {
        let size = [width, height].map(|size| size * (1.0 + 2.0 * MARGIN));
        let cell = [size[0] / CELLS[0] as f32, size[1] / CELLS[1] as f32];
        let step_from = |at: GridPoint, across: f32, up: f32| {
            let step = surface.frame(at).step(across, up)?;
            Some(GridPoint {
                row: at.row + step.row,
                column: at.column + step.column,
            })
        };
        // Up and down the plate's centre line, then across from each node.
        let half = [CELLS[0] / 2, CELLS[1] / 2];
        let mut spine = vec![center; CELLS[1] + 1];
        for k in 1..=half[1] {
            spine[half[1] + k] = step_from(spine[half[1] + k - 1], 0.0, cell[1])?;
            spine[half[1] - k] = step_from(spine[half[1] - k + 1], 0.0, -cell[1])?;
        }
        let mut nodes = Vec::with_capacity((CELLS[0] + 1) * (CELLS[1] + 1));
        for middle in spine {
            let mut row = vec![middle; CELLS[0] + 1];
            for k in 1..=half[0] {
                row[half[0] + k] = step_from(row[half[0] + k - 1], cell[0], 0.0)?;
                row[half[0] - k] = step_from(row[half[0] - k + 1], -cell[0], 0.0)?;
            }
            nodes.extend(row);
        }
        Some(Self {
            corner: [-size[0] * 0.5, -size[1] * 0.5],
            cell,
            nodes,
        })
    }

    /// The grid point at `x` across and `y` up the plate from its centre,
    /// continued linearly past the lattice.
    pub(super) fn at(&self, x: f32, y: f32) -> GridPoint {
        let locate = |value: f32, axis: usize| {
            let t = (value - self.corner[axis]) / self.cell[axis];
            let cell = (t.floor() as i64).clamp(0, CELLS[axis] as i64 - 1) as usize;
            (cell, t - cell as f32)
        };
        let ((i, s), (j, t)) = (locate(x, 0), locate(y, 1));
        let node = |i: usize, j: usize| self.nodes[j * (CELLS[0] + 1) + i];
        let lerp = |a: GridPoint, b: GridPoint, t: f32| GridPoint {
            row: a.row + (b.row - a.row) * t,
            column: a.column + (b.column - a.column) * t,
        };
        lerp(
            lerp(node(i, j), node(i + 1, j), s),
            lerp(node(i, j + 1), node(i + 1, j + 1), s),
            t,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::surface::tests::uneven_cylinder;
    use super::*;

    #[test]
    fn a_plate_keeps_its_size_where_the_grid_crowds() {
        let (grid, positions, normals) = uneven_cylinder(0.05, 0.3, 13, 64, 0.8);
        let surface = GridSurface::base(&grid, &positions, &normals).unwrap();
        let [width, height] = [0.04, 0.08];
        for column in [3.0, 20.0, 32.0, 50.0] {
            let center = GridPoint { row: 6.0, column };
            let map = TileMap::walk(&surface, center, width, height).unwrap();
            let point = |x: f32, y: f32| surface.point(map.at(x, y));
            let up = (point(0.0, height * 0.5) - point(0.0, -height * 0.5)).length();
            assert!((up - height).abs() < height * 0.02, "{column}: {up}");
            // Across a cylinder the chord is shorter than the arc.
            let across = (point(width * 0.5, 0.0) - point(-width * 0.5, 0.0)).length();
            let chord = 2.0 * 0.05 * (width / (2.0 * 0.05)).sin();
            assert!((across - chord).abs() < width * 0.03, "{column}: {across}");
            assert!((point(0.0, 0.0) - surface.point(center)).length() < 1e-6);
        }
    }
}
