//! The cord lacing a piece's plates together, as tubes through their holes.
//!
//! Every row is bound by a running cord through each pair of holes: across
//! the face of each plate from one hole to the other, then behind it to the
//! next plate. A scale's holes are near its top, where the row above covers
//! the cord. Each lamellar lame is also hung from the row above by cords
//! that leave the upper lame's lowest holes, run down over its face, turn
//! under its foot and pass back up behind it into the top holes of the lames
//! below, so the lacing shows over every row.
//!
//! A cord's points lie on the piece's grids like the plates, so the lacing
//! follows every morph with them.

use std::f32::consts::TAU;
use std::ops::Range;

use fabelgeist_math::vector::Vec3;

use super::embedded::{Embedded, EmbeddedMesh};
use super::layout::{Layout, TileRow};
use super::surface::{GridPoint, GridSurface};
use crate::{ConstructionError, Lacing, Plate, material::Metal};

/// Sides of a cord's section.
const SIDES: usize = 6;
/// The longest straight run of a cord, metres, so that it follows the
/// surface between holes.
const STEP: f32 = 0.006;

/// How the plates are laced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Style {
    Lamellar,
    Scale,
}

/// A point of a cord: a place on a grid and its height off the surface.
#[derive(Clone, Copy, Debug)]
struct CordPoint {
    grid: usize,
    at: GridPoint,
    height: f32,
}

/// One cord through the plates.
#[derive(Clone, Debug, Default)]
struct Cord {
    points: Vec<CordPoint>,
    /// Whether it closes on itself, around a closed row.
    closed: bool,
}

/// What laying cords needs of the piece.
pub(super) struct Lacer<'a> {
    pub layout: &'a Layout,
    pub surfaces: &'a [GridSurface<'a>],
    pub plate: &'a Plate,
    pub lacing: &'a Lacing,
}

impl Lacer<'_> {
    /// Every cord of the piece, as one mesh.
    pub(super) fn mesh(&self, style: Style) -> Result<EmbeddedMesh, ConstructionError> {
        let pairs = self.plate.hole_pairs as usize;
        let mut mesh = EmbeddedMesh::default();
        for rows in &self.layout.rows {
            for (index, row) in rows.iter().enumerate() {
                for pair in 0..pairs {
                    for run in &row.runs {
                        self.tube(&mut mesh, &self.binding(run.clone(), row.closed, pair)?)?;
                    }
                }
                if style == Style::Lamellar
                    && let Some(below) = rows.get(index + 1)
                {
                    for tile in row.tiles() {
                        for hole in self.pair(tile, pairs - 1) {
                            self.tube(&mut mesh, &self.hanger(tile, hole, below)?)?;
                        }
                    }
                }
            }
        }
        Ok(mesh)
    }

    /// A point `z` metres out of plate `tile`'s middle, at `x` across and `y`
    /// up it.
    fn on_tile(&self, tile: usize, [x, y, z]: [f32; 3]) -> Result<CordPoint, ConstructionError> {
        let tile = &self.layout.tiles[tile];
        Ok(CordPoint {
            grid: tile.grid,
            at: tile.map.at(x, y),
            height: z + self.layout.rise(tile.grid, self.plate, y),
        })
    }

    fn front(&self) -> f32 {
        self.plate.thickness * 0.5 + self.lacing.radius
    }

    /// Through `hole` on `tile`, from behind the plate to in front of it;
    /// `leaving` reverses it.
    fn through(
        &self,
        tile: usize,
        hole: [f32; 2],
        leaving: bool,
    ) -> Result<[CordPoint; 2], ConstructionError> {
        let [x, y] = hole;
        let back = self.on_tile(tile, [x, y, -self.front()])?;
        let front = self.on_tile(tile, [x, y, self.front()])?;
        Ok(if leaving {
            [front, back]
        } else {
            [back, front]
        })
    }

    /// The pair's holes on a plate, in the order a row's cord meets them.
    fn pair(&self, tile: usize, pair: usize) -> [[f32; 2]; 2] {
        let holes = self.plate.holes();
        let [left, right] = [holes[2 * pair], holes[2 * pair + 1]];
        let grid = self.layout.tiles[tile].grid;
        if self.surfaces[grid].orientation().across > 0.0 {
            [left, right]
        } else {
            [right, left]
        }
    }

    /// The running cord binding a run of neighbouring plates through `pair`,
    /// closing on itself around a whole cyclic row.
    fn binding(
        &self,
        run: Range<usize>,
        closed: bool,
        pair: usize,
    ) -> Result<Cord, ConstructionError> {
        let mut cord = Cord {
            closed: closed && run.len() > 1,
            ..Cord::default()
        };
        for tile in run {
            let [entry, exit] = self.pair(tile, pair);
            let [back, front] = self.through(tile, entry, false)?;
            self.extend(&mut cord, back);
            self.extend(&mut cord, front);
            let [front, back] = self.through(tile, exit, true)?;
            self.extend(&mut cord, front);
            self.extend(&mut cord, back);
        }
        if cord.closed {
            let first = cord.points[0];
            self.extend(&mut cord, first);
            cord.points.pop();
        }
        Ok(cord)
    }

    /// The cord from `hole` on the upper plate `tile` to the nearest top
    /// hole of the row below.
    fn hanger(
        &self,
        tile: usize,
        hole: [f32; 2],
        below: &TileRow,
    ) -> Result<Cord, ConstructionError> {
        let [x, _] = hole;
        let foot = -self.plate.height * 0.5;
        let radius = self.lacing.radius;
        let room = -self.plate.thickness * 0.5 - self.layout.room * 0.5;
        let upper = self.through(tile, hole, false)?;
        let turn = [
            self.on_tile(tile, [x, foot + radius, self.front()])?,
            self.on_tile(tile, [x, foot - radius, 0.0])?,
            self.on_tile(tile, [x, foot + radius, room])?,
        ];
        let start = self.position(upper[1]);
        let mut nearest: Option<(f32, usize, [f32; 2])> = None;
        for lower in below.tiles() {
            for hole in self.pair(lower, 0) {
                let [_, front] = self.through(lower, hole, false)?;
                let distance = (self.position(front) - start).length();
                if nearest.is_none_or(|(best, ..)| distance < best) {
                    nearest = Some((distance, lower, hole));
                }
            }
        }
        let (_, lower, lower_hole) = nearest.ok_or(ConstructionError::DegenerateSurface)?;
        let [front, back] = self.through(lower, lower_hole, true)?;
        let mut cord = Cord::default();
        for point in upper.into_iter().chain(turn).chain([front, back]) {
            self.extend(&mut cord, point);
        }
        Ok(cord)
    }

    fn position(&self, point: CordPoint) -> Vec3 {
        let frame = self.surfaces[point.grid].frame(point.at);
        frame.origin + frame.out * point.height
    }

    /// Continue `cord` to `point`, in steps short enough to follow the
    /// surface.
    fn extend(&self, cord: &mut Cord, point: CordPoint) {
        let Some(&last) = cord.points.last() else {
            cord.points.push(point);
            return;
        };
        let distance = (self.position(point) - self.position(last)).length();
        let steps = (distance / STEP).ceil().max(1.0) as usize;
        let columns = self.surfaces[point.grid].grid().columns as f32;
        let mut turn = point.at.column - last.at.column;
        if self.surfaces[point.grid].grid().cyclic {
            turn -= (turn / columns).round() * columns;
        }
        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            cord.points.push(CordPoint {
                grid: point.grid,
                at: GridPoint {
                    row: last.at.row + (point.at.row - last.at.row) * t,
                    column: last.at.column + turn * t,
                },
                height: last.height + (point.height - last.height) * t,
            });
        }
    }

    /// Wrap `cord` in a tube of the lacing's radius.
    fn tube(&self, mesh: &mut EmbeddedMesh, cord: &Cord) -> Result<(), ConstructionError> {
        let count = cord.points.len();
        if count < 2 {
            return Ok(());
        }
        let positions = cord
            .points
            .iter()
            .map(|point| self.position(*point))
            .collect::<Vec<_>>();
        let tangents = (0..count)
            .map(|i| {
                let (previous, next) = if cord.closed {
                    ((i + count - 1) % count, (i + 1) % count)
                } else {
                    (i.saturating_sub(1), (i + 1).min(count - 1))
                };
                (positions[next] - positions[previous]).normalize()
            })
            .collect::<Vec<_>>();
        let first = mesh.vertices.len() as u32;
        let mut length = 0.0;
        for (i, (point, tangent)) in cord.points.iter().zip(&tangents).enumerate() {
            if i > 0 {
                length += (positions[i] - positions[i - 1]).length();
            }
            let frame = self.surfaces[point.grid].frame(point.at);
            let tangent = *tangent;
            let lifted = frame.out - tangent * frame.out.dot(tangent);
            let u = if lifted.length() > 0.1 {
                lifted.normalize()
            } else {
                frame.across
            };
            let v = tangent.cross(u);
            for side in 0..SIDES {
                let angle = side as f32 * TAU / SIDES as f32;
                let radial = u * angle.cos() + v * angle.sin();
                let offset = radial * self.lacing.radius;
                let [across, up, out] = frame.local(offset);
                let step = frame
                    .step(across, up)
                    .ok_or(ConstructionError::DegenerateSurface)?;
                mesh.vertices.push(Embedded {
                    grid: point.grid,
                    at: GridPoint {
                        row: point.at.row + step.row,
                        column: point.at.column + step.column,
                    },
                    height: point.height + out,
                    normal: frame.local(radial),
                    texcoord: [length * Metal::TILES_PER_METRE, side as f32 / SIDES as f32],
                });
            }
        }
        let rings = if cord.closed { count } else { count - 1 };
        let corner =
            |ring: usize, side: usize| first + ((ring % count) * SIDES + side % SIDES) as u32;
        for ring in 0..rings {
            for side in 0..SIDES {
                let [a, b] = [corner(ring, side), corner(ring, side + 1)];
                let [c, d] = [corner(ring + 1, side), corner(ring + 1, side + 1)];
                mesh.indices.extend([a, b, c, b, d, c]);
            }
        }
        if !cord.closed {
            for (ring, facing) in [(0, -1.0), (count - 1, 1.0)] {
                let tangent = tangents[ring] * facing;
                let point = cord.points[ring];
                let center = mesh.vertices.len() as u32;
                mesh.vertices.push(Embedded {
                    grid: point.grid,
                    at: point.at,
                    height: point.height,
                    normal: self.surfaces[point.grid].frame(point.at).local(tangent),
                    texcoord: mesh.vertices[corner(ring, 0) as usize].texcoord,
                });
                for side in 0..SIDES {
                    let [a, b] = [corner(ring, side), corner(ring, side + 1)];
                    mesh.indices.extend(if facing > 0.0 {
                        [center, a, b]
                    } else {
                        [center, b, a]
                    });
                }
            }
        }
        Ok(())
    }
}
