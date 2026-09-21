//! What the host decides of a plate armor before the device builds it.
//!
//! Everything here depends only on the design and is small: which parts
//! exist and what they are called, where each tile of a lamellar or scale
//! construction sits, and the source triangles every part is cut from -- the
//! template tile (whose rivet holes are cut by CSG), the fauld board, and the
//! breastplate grid's cell corners. The device evaluates, subdivides, maps
//! and welds every triangle.

use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};

use crate::{Armor, Construction, csg::Polygon, mesh};

/// Breastplate grid cells across and down.
pub(super) const GRID_COLUMNS: i32 = 32;
pub(super) const GRID_ROWS: i32 = 24;
/// Subdivision levels of a solid fauld board, so that it follows the curved
/// surface.
const FAULD_BOARD_LEVELS: u32 = 4;
/// Standoff between stacked breastplate tile rows, beyond the plate
/// thickness.
const TILE_ROW_STANDOFF: f32 = 0.0006;
/// Standoff between stacked fauld layers, beyond the plate thickness.
const FAULD_LAYER_STANDOFF: f32 = 0.001;
/// Tolerance for a tile or row reaching past the neckline or fauld bottom.
const TRIM_TOLERANCE: f32 = 0.001;
/// How far across the half width the neckline curve reaches.
const NECKLINE_REACH: f32 = 0.325;

/// A source triangle: three corners of nine floats. Grid corners hold a cell
/// column, row and face side instead of a point.
pub(super) type SourceTriangle = [f32; 9];

/// One placement of a run of source triangles into a part.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct Instance {
    pub first_source: u32,
    pub source_count: u32,
    /// Midpoint subdivisions of each source triangle.
    pub levels: u32,
    /// The first output triangle; outputs are in instance order.
    pub first_output: u32,
    pub part: u32,
    /// Nonzero when the sources are breastplate grid corners.
    pub grid: u32,
    pub pad: [u32; 2],
    pub offset: [f32; 3],
    pub flare: f32,
}

/// Words of an [`Instance`], for the kernel that reads them.
pub(super) const INSTANCE_WORDS: usize = std::mem::size_of::<Instance>() / 4;

/// One part of the armor and the output triangles it owns.
#[derive(Clone, Debug)]
pub(super) struct PlannedPart {
    pub name: String,
    pub first_triangle: u32,
    pub triangle_count: u32,
}

/// The whole armor as the device will build it.
#[derive(Debug, Default)]
pub(super) struct Plan {
    pub sources: Vec<SourceTriangle>,
    pub instances: Vec<Instance>,
    pub parts: Vec<PlannedPart>,
}

/// Where a run of source triangles sits in [`Plan::sources`].
#[derive(Clone, Copy)]
struct Sources {
    first: u32,
    count: u32,
}

/// Template tiles already cut, by the design they were cut for. Cutting the
/// rivet holes is the slow part of the host's work, and the editor rebuilds
/// the same design many times while other controls move.
#[derive(Default)]
pub(super) struct TileCache {
    tiles: Mutex<Vec<CachedTile>>,
}

/// A cut template tile and the design key it was cut for.
type CachedTile = (Vec<u32>, Arc<Vec<SourceTriangle>>);

/// Designs a [`TileCache`] remembers.
const CACHED_TILES: usize = 8;

impl TileCache {
    fn get(&self, a: &Armor) -> Arc<Vec<SourceTriangle>> {
        let p = &a.plate;
        let key = [
            p.gap,
            p.bevel,
            p.width,
            p.height,
            p.roundness,
            p.overlap,
            p.stagger,
            p.hole_radius,
            a.thickness,
        ]
        .iter()
        .map(|value| value.to_bits())
        .chain([p.hole_pairs])
        .collect::<Vec<_>>();
        let mut tiles = self.tiles.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(index) = tiles.iter().position(|(cached, _)| *cached == key) {
            let entry = tiles.remove(index);
            let tile = entry.1.clone();
            tiles.push(entry);
            return tile;
        }
        let tile = Arc::new(fan(&mesh::tile(a)));
        if tiles.len() == CACHED_TILES {
            tiles.remove(0);
        }
        tiles.push((key, tile.clone()));
        tile
    }
}

/// Each polygon as a fan of triangles from its first corner, in order.
fn fan(polygons: &[Polygon]) -> Vec<SourceTriangle> {
    let mut triangles = Vec::new();
    for polygon in polygons {
        let corner = |i: usize| {
            let p = polygon.vertices[i].position;
            [p.x, p.y, p.z]
        };
        for i in 1..polygon.vertices.len() - 1 {
            let [a, b, c] = [corner(0), corner(i), corner(i + 1)];
            triangles.push([a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]]);
        }
    }
    triangles
}

/// The breastplate grid: every cell's front and back, and the walls around
/// the outside, as cell corners `(column, row, side)`.
fn grid() -> Vec<SourceTriangle> {
    let mut triangles = Vec::new();
    let mut quad = |corners: [(i32, i32, f32); 4]| {
        let corner = |k: usize| {
            let (i, j, side) = corners[k];
            [i as f32, j as f32, side]
        };
        for [a, b, c] in [[0, 1, 2], [0, 2, 3]].map(|t| t.map(corner)) {
            triangles.push([a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]]);
        }
    };
    let (nx, ny) = (GRID_COLUMNS, GRID_ROWS);
    for x in 0..nx {
        for y in 0..ny {
            let z = 1.0;
            quad([(x, y, z), (x + 1, y, z), (x + 1, y + 1, z), (x, y + 1, z)]);
            quad([
                (x, y, -z),
                (x, y + 1, -z),
                (x + 1, y + 1, -z),
                (x + 1, y, -z),
            ]);
            if x == 0 {
                quad([(x, y, z), (x, y + 1, z), (x, y + 1, -z), (x, y, -z)]);
            }
            if x == nx - 1 {
                quad([
                    (x + 1, y + 1, z),
                    (x + 1, y, z),
                    (x + 1, y, -z),
                    (x + 1, y + 1, -z),
                ]);
            }
            if y == 0 {
                quad([(x + 1, y, z), (x, y, z), (x, y, -z), (x + 1, y, -z)]);
            }
            if y == ny - 1 {
                quad([
                    (x, y + 1, z),
                    (x + 1, y + 1, z),
                    (x + 1, y + 1, -z),
                    (x, y + 1, -z),
                ]);
            }
        }
    }
    triangles
}

impl Plan {
    /// Lay out `a`'s parts, tiles and source triangles.
    pub(super) fn new(a: &Armor, tiles: &TileCache) -> Result<Self, String> {
        a.validate()?;
        let mut plan = Self::default();
        let mut tile = None;
        let mut tile_sources =
            |plan: &mut Self| *tile.get_or_insert_with(|| plan.push_sources(&tiles.get(a)));
        if a.construction == Construction::Solid {
            let sources = plan.push_sources(&grid());
            plan.push_part("Breastplate".into());
            plan.push_instance(sources, 0, true, [0.0; 3], 0.0);
        } else {
            let sources = tile_sources(&mut plan);
            plan.breastplate_tiles(a, sources);
        }
        let mut board = None;
        let mut bottom = 0.0;
        let mut flare = 0.0;
        for i in 0..a.fauld.layer_count as usize {
            bottom -= a.fauld.layer_height * (1.0 - a.fauld.overlap);
            flare += a.fauld.flare;
            let z =
                (a.fauld.layer_count as usize - i) as f32 * (a.thickness + FAULD_LAYER_STANDOFF);
            plan.push_part(format!("Fauld layer {}", i + 1));
            if a.fauld.construction == Construction::Solid {
                let sources = *board.get_or_insert_with(|| {
                    let outline = [
                        [-a.width * 0.5, 0.0],
                        [a.width * 0.5, 0.0],
                        [a.width * 0.5, a.fauld.layer_height],
                        [-a.width * 0.5, a.fauld.layer_height],
                    ];
                    plan.push_sources(&fan(&mesh::extrude(&outline, a.thickness)))
                });
                plan.push_instance(sources, FAULD_BOARD_LEVELS, false, [0.0, bottom, z], flare);
            } else {
                let sources = tile_sources(&mut plan);
                plan.fauld_tiles(a, sources, bottom, z, flare);
            }
        }
        Ok(plan)
    }

    /// Output triangles of every part.
    pub(super) fn triangle_count(&self) -> u32 {
        self.parts
            .last()
            .map_or(0, |part| part.first_triangle + part.triangle_count)
    }

    fn push_sources(&mut self, triangles: &[SourceTriangle]) -> Sources {
        let first = self.sources.len() as u32;
        self.sources.extend_from_slice(triangles);
        Sources {
            first,
            count: triangles.len() as u32,
        }
    }

    fn push_part(&mut self, name: String) {
        self.parts.push(PlannedPart {
            name,
            first_triangle: self.triangle_count(),
            triangle_count: 0,
        });
    }

    /// Place `sources` in the last part.
    fn push_instance(
        &mut self,
        sources: Sources,
        levels: u32,
        grid: bool,
        offset: [f32; 3],
        flare: f32,
    ) {
        let part = self.parts.len() - 1;
        let first_output = self.triangle_count();
        self.instances.push(Instance {
            first_source: sources.first,
            source_count: sources.count,
            levels,
            first_output,
            part: part as u32,
            grid: grid.into(),
            pad: [0; 2],
            offset,
            flare,
        });
        self.parts[part].triangle_count += sources.count << (2 * levels);
    }

    /// The breastplate's rows of tiles, trimmed at the neckline and arm
    /// openings, each its own part.
    fn breastplate_tiles(&mut self, a: &Armor, tile: Sources) {
        let p = &a.plate;
        let rows = ((a.height - p.height) / (p.height * (1.0 - p.overlap)))
            .ceil()
            .max(0.0) as u32
            + 1;
        for row in 0..rows {
            let y = a.height
                - p.height * 0.5
                - row as f32 * (a.height - p.height) / (rows - 1).max(1) as f32;
            let usable = a.width - 2.0 * a.arm_cut * (y / a.height).powi(4);
            let pitch = p.width + p.gap;
            let cols = ((usable + p.gap) / pitch).floor().max(1.0) as u32;
            let stagger = if row % 2 == 1 { p.stagger } else { 0.0 };
            let count = if stagger > 0.0 && cols > 1 {
                cols - 1
            } else {
                cols
            };
            for col in 0..count {
                let x = (col as f32 - (cols - 1) as f32 * 0.5 + stagger) * pitch;
                let top = a.height
                    - a.neck * (1.0 - (x.abs() / (a.width * NECKLINE_REACH)).min(1.0).powi(2));
                if y + p.height * 0.5 > top + TRIM_TOLERANCE {
                    continue;
                }
                let z = (rows - row) as f32 * (a.thickness + TILE_ROW_STANDOFF);
                self.push_part(format!("Plate {}:{}", row + 1, col + 1));
                self.push_instance(tile, 0, false, [x, y, z], 0.0);
            }
        }
    }

    /// One fauld layer's overlapping rows of tiles, in the last part.
    fn fauld_tiles(&mut self, a: &Armor, tile: Sources, bottom: f32, z: f32, flare: f32) {
        let p = &a.plate;
        let rows = (a.fauld.layer_height / (p.height * (1.0 - p.overlap)))
            .ceil()
            .max(1.0) as u32;
        let cols = ((a.width + p.gap) / (p.width + p.gap)).floor().max(1.0) as u32;
        for row in 0..rows {
            let y = bottom + a.fauld.layer_height
                - p.height * 0.5
                - row as f32 * p.height * (1.0 - p.overlap);
            if y - p.height * 0.5 < bottom - TRIM_TOLERANCE {
                continue;
            }
            let stagger = if row % 2 == 1 { p.stagger } else { 0.0 };
            for col in 0..cols {
                let x = (col as f32 - (cols - 1) as f32 * 0.5 + stagger) * (p.width + p.gap);
                self.push_instance(tile, 0, false, [x, y, z], flare);
            }
        }
    }
}
