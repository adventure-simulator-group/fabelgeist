//! A plate's outer face as a surface over its grid, on one realization of the
//! piece: its base fit or one of its morphs.
//!
//! A generator spaces a grid's rows and columns as its shape needs -- densely
//! across a flute, sparsely over a plain back -- so a grid index is no
//! measure of distance. A point is therefore addressed in even coordinates:
//! a row or column index rescaled so that each unit covers the same distance,
//! averaged over the grid, as measured on the base fit. The same coordinates
//! then name the same place on every morph.
//!
//! The surface interpolates the grid bilinearly and continues its border
//! cells linearly past the edge, so a plate may overhang an open border
//! slightly; a cyclic grid wraps around its columns. The frame at a point is
//! the surface's outward normal and two tangents: `across`, along the
//! columns, and `up`, towards the grid's top row, which the base fit decides
//! once.

use std::sync::Arc;

use fabelgeist_math::vector::Vec3;

use crate::SurfaceGrid;

/// Even units either side of a point its tangents are differenced over:
/// half a cell, so a frame turns smoothly across cell borders.
const TANGENT_STEP: f32 = 0.5;

/// A position in a grid's even coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridPoint {
    pub row: f32,
    pub column: f32,
}

/// Which way a grid faces, decided on the base fit and kept for every morph
/// so that frames correspond.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Orientation {
    /// `1` when `d_column × d_row` points out of the piece, else `-1`.
    outward: f32,
    /// `1` when `across` runs with increasing columns, else `-1`.
    pub across: f32,
    /// Whether rows are numbered from the top of the piece down.
    pub downward: bool,
}

/// Each grid index's distance from the first, metres, averaged over the
/// grid on the base fit: rows along the columns, and columns along the rows,
/// a cyclic grid's closing span last.
#[derive(Debug)]
struct Spacing {
    rows: Vec<f32>,
    columns: Vec<f32>,
}

impl Spacing {
    /// Metres per even unit.
    fn pitch(distances: &[f32]) -> f32 {
        distances[distances.len() - 1] / (distances.len() - 1) as f32
    }
}

/// The surface's frame at a point.
#[derive(Clone, Copy, Debug)]
pub(super) struct Frame {
    pub origin: Vec3,
    pub across: Vec3,
    pub up: Vec3,
    pub out: Vec3,
    d_row: Vec3,
    d_column: Vec3,
}

impl Frame {
    /// The step in even coordinates that moves `across` and `up` metres along
    /// the surface, to first order.
    pub(super) fn step(&self, across: f32, up: f32) -> Option<GridPoint> {
        let d = self.across * across + self.up * up;
        let (a, b) = (self.d_row, self.d_column);
        let [aa, ab, bb] = [a.dot(a), a.dot(b), b.dot(b)];
        let determinant = aa * bb - ab * ab;
        if determinant.is_nan() || determinant <= f32::EPSILON * aa * bb {
            return None;
        }
        let [da, db] = [a.dot(d), b.dot(d)];
        Some(GridPoint {
            row: (bb * da - ab * db) / determinant,
            column: (aa * db - ab * da) / determinant,
        })
    }

    /// A direction given in the frame, in the piece's space.
    pub(super) fn direction(&self, [across, up, out]: [f32; 3]) -> Vec3 {
        (self.across * across + self.up * up + self.out * out).normalize()
    }

    /// A direction in the piece's space, in the frame.
    pub(super) fn local(&self, direction: Vec3) -> [f32; 3] {
        [
            direction.dot(self.across),
            direction.dot(self.up),
            direction.dot(self.out),
        ]
    }
}

/// One grid on one realization of the piece.
#[derive(Clone, Debug)]
pub(super) struct GridSurface<'a> {
    grid: &'a SurfaceGrid,
    positions: &'a [[f32; 3]],
    orientation: Orientation,
    spacing: Arc<Spacing>,
}

impl<'a> GridSurface<'a> {
    /// A grid too small to span a cell is no surface.
    pub(super) fn is_spanning(grid: &SurfaceGrid) -> bool {
        let columns = if grid.cyclic { 3 } else { 2 };
        grid.rows >= 2
            && grid.columns >= columns
            && grid.vertices.len() == (grid.rows * grid.columns) as usize
    }

    /// The grid on the piece's base fit, measured, and oriented by its
    /// shading normals and the world's up. `None` when it has no extent.
    pub(super) fn base(
        grid: &'a SurfaceGrid,
        positions: &'a [[f32; 3]],
        normals: &[[f32; 3]],
    ) -> Option<Self> {
        let vertex = |row: u32, column: u32| {
            Vec3::from_array(positions[grid.vertex(row, column % grid.columns) as usize])
        };
        let mean =
            |steps: &mut dyn Iterator<Item = f32>, count: u32| steps.sum::<f32>() / count as f32;
        let mut rows = vec![0.0];
        for row in 1..grid.rows {
            let step = mean(
                &mut (0..grid.columns).map(|c| (vertex(row, c) - vertex(row - 1, c)).length()),
                grid.columns,
            );
            rows.push(rows[row as usize - 1] + step);
        }
        let spans = if grid.cyclic {
            grid.columns
        } else {
            grid.columns - 1
        };
        let mut columns = vec![0.0];
        for column in 1..=spans {
            let step = mean(
                &mut (0..grid.rows).map(|r| (vertex(r, column) - vertex(r, column - 1)).length()),
                grid.rows,
            );
            columns.push(columns[column as usize - 1] + step);
        }
        let extent = |distances: &[f32]| distances[distances.len() - 1];
        if !(extent(&rows) > 0.0 && extent(&columns) > 0.0) {
            return None;
        }
        let mut surface = Self {
            grid,
            positions,
            orientation: Orientation {
                outward: 1.0,
                across: 1.0,
                downward: true,
            },
            spacing: Arc::new(Spacing { rows, columns }),
        };
        let at_vertices = || {
            (0..grid.rows).flat_map(move |row| (0..grid.columns).map(move |column| (row, column)))
        };
        let (mut outward, mut downward) = (0.0f32, 0.0f32);
        for (row, column) in at_vertices() {
            let frame = surface.frame(surface.at_vertex(row, column));
            let normal = Vec3::from_array(normals[grid.vertex(row, column) as usize]);
            outward += frame.out.dot(normal);
            downward -= frame.d_row.dot(Vec3::up());
        }
        if !(outward != 0.0 && outward.is_finite()) {
            return None;
        }
        surface.orientation.outward = outward.signum();
        surface.orientation.downward = downward >= 0.0;
        let mut up = 0.0f32;
        for (row, column) in at_vertices() {
            let frame = surface.frame(surface.at_vertex(row, column));
            let top = if surface.orientation.downward {
                -frame.d_row
            } else {
                frame.d_row
            };
            up += frame.up.dot(top);
        }
        surface.orientation.across = if up >= 0.0 { 1.0 } else { -1.0 };
        Some(surface)
    }

    /// The same grid, measured and oriented as on the base, on another
    /// realization.
    pub(super) fn realized(&self, positions: &'a [[f32; 3]]) -> Self {
        Self {
            positions,
            ..self.clone()
        }
    }

    pub(super) fn grid(&self) -> &SurfaceGrid {
        self.grid
    }

    pub(super) fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Metres from the first row to the last, averaged over the columns.
    pub(super) fn length(&self) -> f32 {
        self.spacing.rows[self.spacing.rows.len() - 1]
    }

    /// The even row `metres` along the grid from its first row.
    pub(super) fn row_at(&self, metres: f32) -> f32 {
        metres / Spacing::pitch(&self.spacing.rows)
    }

    /// Where a grid vertex lies in even coordinates.
    fn at_vertex(&self, row: u32, column: u32) -> GridPoint {
        GridPoint {
            row: self.spacing.rows[row as usize] / Spacing::pitch(&self.spacing.rows),
            column: self.spacing.columns[column as usize] / Spacing::pitch(&self.spacing.columns),
        }
    }

    /// The fractional grid indices of a point.
    fn indices(&self, at: GridPoint) -> [f32; 2] {
        let row = index_at(
            &self.spacing.rows,
            at.row * Spacing::pitch(&self.spacing.rows),
        );
        let columns = &self.spacing.columns;
        let spans = (columns.len() - 1) as f32;
        let column = if self.grid.cyclic {
            at.column.rem_euclid(spans)
        } else {
            at.column
        };
        [row, index_at(columns, column * Spacing::pitch(columns))]
    }

    fn vertex(&self, row: u32, column: u32) -> Vec3 {
        Vec3::from_array(self.positions[self.grid.vertex(row, column) as usize])
    }

    /// The cell a point lies in or beyond, and its place across it.
    fn cell(&self, at: GridPoint) -> ([u32; 2], [u32; 2], [f32; 2]) {
        let [row, column] = self.indices(at);
        let rows = self.grid.rows;
        let columns = self.grid.columns;
        let first_row = (row.floor() as i64).clamp(0, rows as i64 - 2) as u32;
        let s = row - first_row as f32;
        if self.grid.cyclic {
            let first = (column.floor() as u32).min(columns - 1);
            let t = column - first as f32;
            (
                [first_row, first_row + 1],
                [first, (first + 1) % columns],
                [s, t],
            )
        } else {
            let first = (column.floor() as i64).clamp(0, columns as i64 - 2) as u32;
            let t = column - first as f32;
            ([first_row, first_row + 1], [first, first + 1], [s, t])
        }
    }

    pub(super) fn point(&self, at: GridPoint) -> Vec3 {
        let ([r0, r1], [c0, c1], [s, t]) = self.cell(at);
        let lower = self.vertex(r0, c0).lerp(self.vertex(r0, c1), t);
        let upper = self.vertex(r1, c0).lerp(self.vertex(r1, c1), t);
        lower.lerp(upper, s)
    }

    /// The grid vertices a point is blended from, and their weights, the
    /// point held inside its cell.
    pub(super) fn corners(&self, at: GridPoint) -> [(u32, f32); 4] {
        let ([r0, r1], [c0, c1], [s, t]) = self.cell(at);
        let [s, t] = [s.clamp(0.0, 1.0), t.clamp(0.0, 1.0)];
        [
            (self.grid.vertex(r0, c0), (1.0 - s) * (1.0 - t)),
            (self.grid.vertex(r0, c1), (1.0 - s) * t),
            (self.grid.vertex(r1, c0), s * (1.0 - t)),
            (self.grid.vertex(r1, c1), s * t),
        ]
    }

    pub(super) fn frame(&self, at: GridPoint) -> Frame {
        let shifted = |row: f32, column: f32| {
            self.point(GridPoint {
                row: at.row + row,
                column: at.column + column,
            })
        };
        let h = TANGENT_STEP;
        let d_row = (shifted(h, 0.0) - shifted(-h, 0.0)) * (0.5 / h);
        let d_column = (shifted(0.0, h) - shifted(0.0, -h)) * (0.5 / h);
        let out = d_column.cross(d_row).normalize() * self.orientation.outward;
        let across = (d_column - out * d_column.dot(out)).normalize() * self.orientation.across;
        Frame {
            origin: self.point(at),
            up: out.cross(across),
            across,
            out,
            d_row,
            d_column,
        }
    }

    /// Metres across even row `row` from its first even column to each
    /// whole one, the closing span of a cyclic grid last.
    pub(super) fn column_distances(&self, row: f32) -> Vec<f32> {
        let point = |column: u32| {
            self.point(GridPoint {
                row,
                column: column as f32,
            })
        };
        let spans = self.spacing.columns.len() as u32 - 1;
        let mut distances = vec![0.0];
        for column in 0..spans {
            let span = (point(column + 1) - point(column)).length();
            distances.push(distances[column as usize] + span);
        }
        distances
    }
}

/// The fractional index at which cumulative `distances` reach `distance`,
/// continued linearly past either end.
pub(super) fn index_at(distances: &[f32], distance: f32) -> f32 {
    let last = distances.len() - 1;
    let span = distances.partition_point(|d| *d <= distance).clamp(1, last);
    let (from, to) = (distances[span - 1], distances[span]);
    let t = if to > from {
        (distance - from) / (to - from)
    } else {
        0.0
    };
    (span - 1) as f32 + t
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// A cylinder of `radius` around the world's up, `rows` rings from
    /// `height` down to zero, as a cyclic grid wound outward.
    pub(in crate::tiling) fn cylinder(
        radius: f32,
        height: f32,
        rows: u32,
        columns: u32,
    ) -> (SurfaceGrid, Vec<[f32; 3]>, Vec<[f32; 3]>) {
        uneven_cylinder(radius, height, rows, columns, 0.0)
    }

    /// A cylinder whose columns crowd towards its front by `crowding`, from
    /// zero (even) towards one.
    pub(in crate::tiling) fn uneven_cylinder(
        radius: f32,
        height: f32,
        rows: u32,
        columns: u32,
        crowding: f32,
    ) -> (SurfaceGrid, Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        for row in 0..rows {
            let y = height * (1.0 - row as f32 / (rows - 1) as f32);
            for column in 0..columns {
                let even = column as f32 * std::f32::consts::TAU / columns as f32;
                let angle = even - crowding * even.sin();
                positions.push([radius * angle.cos(), y, -radius * angle.sin()]);
                normals.push([angle.cos(), 0.0, -angle.sin()]);
            }
        }
        let grid = SurfaceGrid {
            rows,
            columns,
            cyclic: true,
            vertices: (0..rows * columns).collect(),
        };
        (grid, positions, normals)
    }

    #[test]
    fn a_frame_faces_out_and_up_the_piece() {
        let (grid, positions, normals) = cylinder(0.05, 0.2, 9, 32);
        let surface = GridSurface::base(&grid, &positions, &normals).unwrap();
        assert!(surface.orientation().downward);
        for column in [0.0, 7.5, 31.9] {
            let frame = surface.frame(GridPoint { row: 4.0, column });
            let radial = Vec3::new(frame.origin.x, 0.0, frame.origin.z).normalize();
            assert!(frame.out.dot(radial) > 0.99);
            assert!(frame.up.dot(Vec3::up()) > 0.99);
            assert!(frame.across.cross(frame.up).dot(frame.out) > 0.99);
        }
    }

    #[test]
    fn a_step_moves_the_distance_asked_however_the_columns_crowd() {
        for crowding in [0.0, 0.8] {
            let (grid, positions, normals) = uneven_cylinder(0.05, 0.2, 9, 64, crowding);
            let surface = GridSurface::base(&grid, &positions, &normals).unwrap();
            for column in [2.0, 16.0, 32.0, 48.0] {
                let at = GridPoint { row: 4.0, column };
                let frame = surface.frame(at);
                let step = frame.step(0.02, 0.02).unwrap();
                let moved = surface.point(GridPoint {
                    row: at.row + step.row,
                    column: at.column + step.column,
                });
                let offset = moved - frame.origin;
                assert!(
                    (offset.dot(Vec3::up()) - 0.02).abs() < 1e-4,
                    "{crowding} {column}"
                );
                let across = offset.dot(frame.across);
                assert!(
                    (across - 0.02).abs() < 0.002,
                    "{crowding} {column}: {across}"
                );
            }
        }
    }

    #[test]
    fn distances_invert_to_their_index() {
        let distances = [0.0, 1.0, 3.0, 6.0];
        assert_eq!(index_at(&distances, 2.0), 1.5);
        assert_eq!(index_at(&distances, 6.0), 3.0);
        assert_eq!(index_at(&distances, -1.0), -1.0);
        assert_eq!(index_at(&distances, 7.5), 3.5);
    }
}
