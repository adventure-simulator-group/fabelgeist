//! Where every small plate of a piece lies, decided on its base fit.
//!
//! Each grid is covered by rows of plates from its top down, as many as its
//! length along the piece takes at the plate's overlap. Each row takes as
//! many plates as fit across it at that height -- whole ones across an open
//! plate, evenly spaced ones around a closed one -- every other row shifted
//! by the stagger. A plate is tilted along the piece so its top tucks under
//! the row above and its foot lies over the row below, the rows clearing
//! each other by the lacing's room however many there are.
//!
//! A plate keeps its grid position on every morph of the piece, so the
//! plates correspond wherever the body moves the surface.

use std::ops::Range;

use super::surface::{GridPoint, GridSurface, index_at};
use super::tile_map::TileMap;
use crate::{ConstructionError, Plate};

/// Room between overlapping rows of unlaced plates, metres.
const UNLACED_ROOM: f32 = 0.0006;
/// Room around the cord between overlapping rows, beyond its diameter,
/// metres.
const LACING_ROOM_MARGIN: f32 = 0.0004;
/// The closest rows are squeezed, as a share of the pitch the overlap asks.
const LEAST_PITCH: f32 = 0.5;
/// How far a plate may reach past its grid's border, as a share of its own
/// size, before it is left out.
const OVERHANG: f32 = 0.25;
/// Even units a plate's corner may pass its reach by, so that a plate
/// overhanging exactly as far as it may is kept.
const REACH_TOLERANCE: f32 = 1e-3;

/// One placed plate.
#[derive(Clone, Debug)]
pub(super) struct Tile {
    pub grid: usize,
    /// Where the plate's own coordinates land on its grid.
    pub map: TileMap,
}

/// A row of plates across a grid, in column order, as runs of neighbours:
/// a plate reaching past the grid's border is left out, and breaks its run.
#[derive(Clone, Debug)]
pub(super) struct TileRow {
    pub runs: Vec<Range<usize>>,
    /// Whether the row's one run closes around its grid.
    pub closed: bool,
}

impl TileRow {
    pub(super) fn tiles(&self) -> impl Iterator<Item = usize> + '_ {
        self.runs.iter().flat_map(Clone::clone)
    }
}

/// Every plate of a piece.
#[derive(Clone, Debug, Default)]
pub(super) struct Layout {
    pub tiles: Vec<Tile>,
    /// Each grid's rows, from its top down.
    pub rows: Vec<Vec<TileRow>>,
    /// How far each plate's surface rises per metre down it.
    pub slopes: Vec<f32>,
    /// The room between overlapping rows, metres.
    pub room: f32,
}

impl Layout {
    pub(super) fn new(
        surfaces: &[GridSurface],
        plate: &Plate,
        lacing_radius: Option<f32>,
        max_tiles: usize,
    ) -> Result<Self, ConstructionError> {
        let room = lacing_radius.map_or(UNLACED_ROOM, |radius| 2.0 * radius + LACING_ROOM_MARGIN);
        let mut layout = Self {
            room,
            ..Self::default()
        };
        for (grid, surface) in surfaces.iter().enumerate() {
            let pitch = layout.lay_grid(grid, surface, plate)?;
            layout.slopes.push((plate.thickness + room) / pitch);
            if layout.tiles.len() > max_tiles {
                return Err(ConstructionError::TooManyPlates);
            }
        }
        if layout.tiles.is_empty() {
            return Err(ConstructionError::NoPlateFits);
        }
        Ok(layout)
    }

    /// The plate's height above the surface at `y` metres up it: its top
    /// sits a gauge under the surface, its foot lifted by the slope.
    pub(super) fn rise(&self, grid: usize, plate: &Plate, y: f32) -> f32 {
        -plate.thickness * 0.5 + (plate.height * 0.5 - y) * self.slopes[grid]
    }

    /// Lay one grid's rows; returns the pitch between them.
    fn lay_grid(
        &mut self,
        grid: usize,
        surface: &GridSurface,
        plate: &Plate,
    ) -> Result<f32, ConstructionError> {
        let length = surface.length();
        if !(length > 0.0 && length.is_finite()) {
            return Err(ConstructionError::DegenerateSurface);
        }
        let height = plate.height;
        let most_pitch = height * (1.0 - plate.overlap);
        let mut rows = if length > height {
            ((length - height) / most_pitch).ceil() as usize + 1
        } else {
            1
        };
        // Rows squeezed much closer than the overlap asks would have to tilt
        // steeply to clear each other; a row fewer leaves a sliver uncovered
        // at the ends instead.
        if rows > 1 && (length - height) / ((rows - 1) as f32) < most_pitch * LEAST_PITCH {
            rows -= 1;
        }
        let pitch = if rows > 1 {
            (length - height) / (rows - 1) as f32
        } else {
            most_pitch
        };
        let mut grid_rows = Vec::with_capacity(rows);
        for row in 0..rows {
            let from_top = if rows > 1 {
                height * 0.5 + row as f32 * pitch
            } else {
                length * 0.5
            };
            let along = if surface.orientation().downward {
                from_top
            } else {
                length - from_top
            };
            let fraction = surface.row_at(along);
            let stagger = if row % 2 == 1 { plate.stagger } else { 0.0 };
            let (columns, width) = columns(surface, fraction, plate, stagger);
            let bounds = Bounds::new(surface, plate, width);
            let mut runs = Vec::new();
            let mut run = self.tiles.len()..self.tiles.len();
            let mut dropped = false;
            for column in columns {
                let center = GridPoint {
                    row: fraction,
                    column,
                };
                let map = TileMap::walk(surface, center, plate.width, plate.height);
                if let Some(map) = map.filter(|map| bounds.holds(map, plate)) {
                    self.tiles.push(Tile { grid, map });
                    run.end += 1;
                    continue;
                }
                dropped = true;
                if !run.is_empty() {
                    runs.push(run);
                    run = self.tiles.len()..self.tiles.len();
                }
            }
            if !run.is_empty() {
                runs.push(run);
            }
            grid_rows.push(TileRow {
                closed: surface.grid().cyclic && !dropped,
                runs,
            });
        }
        self.rows.push(grid_rows);
        Ok(pitch)
    }
}

/// How far past a grid's borders a plate of one row may reach, in even
/// units.
struct Bounds {
    rows: f32,
    columns: Option<f32>,
    row_overhang: f32,
    column_overhang: f32,
}

impl Bounds {
    /// Around a row `width` metres across. A grid shorter or narrower than
    /// the plate lets it overhang by the difference.
    fn new(surface: &GridSurface, plate: &Plate, width: f32) -> Self {
        let grid = surface.grid();
        let overhang = |size: f32, extent: f32| (OVERHANG * size).max((size - extent) * 0.5);
        let spans = (grid.columns - 1) as f32;
        Self {
            rows: (grid.rows - 1) as f32,
            columns: (!grid.cyclic).then_some(spans),
            row_overhang: surface.row_at(overhang(plate.height, surface.length()))
                + REACH_TOLERANCE,
            column_overhang: overhang(plate.width, width) * spans / width + REACH_TOLERANCE,
        }
    }

    /// Whether every corner of a plate stays within reach of the grid.
    fn holds(&self, map: &TileMap, plate: &Plate) -> bool {
        let [x, y] = [plate.width * 0.5, plate.height * 0.5];
        [[-x, -y], [x, -y], [x, y], [-x, y]]
            .into_iter()
            .all(|[across, up]| {
                let GridPoint { row, column } = map.at(across, up);
                (-self.row_overhang..=self.rows + self.row_overhang).contains(&row)
                    && self.columns.is_none_or(|columns| {
                        (-self.column_overhang..=columns + self.column_overhang).contains(&column)
                    })
            })
    }
}

/// The even columns of the plates across even row `row`, in column order,
/// and the row's width in metres.
fn columns(surface: &GridSurface, row: f32, plate: &Plate, stagger: f32) -> (Vec<f32>, f32) {
    let distances = surface.column_distances(row);
    let width = distances[distances.len() - 1];
    let pitch = plate.width + plate.gap;
    let along = |distance: f32| index_at(&distances, distance);
    if surface.grid().cyclic {
        let count = (width / pitch).ceil().max(1.0) as usize;
        let spacing = width / count as f32;
        let columns = (0..count)
            .map(|i| along((i as f32 + stagger) * spacing))
            .collect();
        return (columns, width);
    }
    let fits = ((width + plate.gap) / pitch).floor().max(1.0) as usize;
    let count = if stagger > 0.0 && fits > 1 {
        fits - 1
    } else {
        fits
    };
    let across = surface.orientation().across;
    let mut columns = (0..count)
        .map(|i| {
            let x = (i as f32 - (fits - 1) as f32 * 0.5 + stagger) * pitch;
            along(width * 0.5 + across * x)
        })
        .collect::<Vec<_>>();
    columns.sort_by(f32::total_cmp);
    (columns, width)
}

#[cfg(test)]
mod tests {
    use super::super::surface::tests::cylinder;
    use super::*;
    use crate::Tiling;

    #[test]
    fn rows_cover_the_piece_and_close_around_it() {
        let (grid, positions, normals) = cylinder(0.05, 0.25, 11, 48);
        let surface = GridSurface::base(&grid, &positions, &normals).unwrap();
        let plate = Tiling::scale().plate;
        let layout = Layout::new(
            std::slice::from_ref(&surface),
            &plate,
            Some(0.001),
            usize::MAX,
        )
        .unwrap();
        let rows = &layout.rows[0];
        // The rows reach from the top to the bottom at no more than the
        // overlap's pitch.
        let pitch = plate.height * (1.0 - plate.overlap);
        assert_eq!(
            rows.len(),
            ((0.25 - plate.height) / pitch).ceil() as usize + 1
        );
        let circumference = std::f32::consts::TAU * 0.05;
        for row in rows {
            assert!(row.closed && row.runs.len() == 1);
            let count = row.tiles().count() as f32;
            assert!(count * (plate.width + plate.gap) >= circumference * 0.98);
            assert!((count - 1.0) * (plate.width + plate.gap) < circumference);
        }
        let origin = |tile: usize| surface.point(layout.tiles[tile].map.at(0.0, 0.0));
        let top = origin(rows[0].runs[0].start).y;
        let bottom = origin(rows[rows.len() - 1].runs[0].start).y;
        assert!((top - (0.25 - plate.height * 0.5)).abs() < 1e-3);
        assert!((bottom - plate.height * 0.5).abs() < 1e-3);
    }

    #[test]
    fn overlapping_rows_clear_each_other_by_the_lacing_room() {
        let (grid, positions, normals) = cylinder(0.05, 0.25, 11, 48);
        let surface = GridSurface::base(&grid, &positions, &normals).unwrap();
        let plate = Tiling::scale().plate;
        let radius = 0.001;
        let layout = Layout::new(
            std::slice::from_ref(&surface),
            &plate,
            Some(radius),
            usize::MAX,
        )
        .unwrap();
        let rows = &layout.rows[0];
        let [upper, lower] = [0, 1].map(|row| {
            surface
                .point(layout.tiles[rows[row].runs[0].start].map.at(0.0, 0.0))
                .y
        });
        let pitch = upper - lower;
        // At the upper plate's foot, its back clears the lower plate's front.
        let foot = -plate.height * 0.5;
        let upper_back = layout.rise(0, &plate, foot) - plate.thickness * 0.5;
        let lower_front = layout.rise(0, &plate, foot + pitch) + plate.thickness * 0.5;
        assert!((upper_back - lower_front - (2.0 * radius + LACING_ROOM_MARGIN)).abs() < 1e-6);
    }

    #[test]
    fn rows_are_never_squeezed_to_a_steep_tilt() {
        let plate = Tiling::scale().plate;
        let most_pitch = plate.height * (1.0 - plate.overlap);
        for extra in [0.05, 0.3, 0.6, 1.2, 2.5] {
            let height = plate.height + extra * most_pitch;
            let (grid, positions, normals) = cylinder(0.05, height, 11, 48);
            let surface = GridSurface::base(&grid, &positions, &normals).unwrap();
            let layout =
                Layout::new(std::slice::from_ref(&surface), &plate, None, usize::MAX).unwrap();
            let steepest = (plate.thickness + UNLACED_ROOM) / (most_pitch * LEAST_PITCH);
            assert!(layout.slopes[0] <= steepest * 1.0001, "{extra}");
        }
    }

    #[test]
    fn a_piece_taking_too_many_plates_is_refused() {
        let (grid, positions, normals) = cylinder(0.05, 0.25, 11, 48);
        let surface = GridSurface::base(&grid, &positions, &normals).unwrap();
        let plate = Tiling::scale().plate;
        assert_eq!(
            Layout::new(std::slice::from_ref(&surface), &plate, None, 10).unwrap_err(),
            ConstructionError::TooManyPlates
        );
    }
}
