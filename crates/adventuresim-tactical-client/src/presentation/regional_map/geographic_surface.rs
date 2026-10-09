//! Geographic positions on the exact triangles emitted by the regional mesh.
use adventuresim_tactical_core::regional_terrain::{REGIONAL_TERRAIN_SIDE, RegionalTerrain};
use adventuresim_world_schema::coordinates::{
    Wgs84CoordinateMicrodegrees,
    terrain_projection::{NativeTerrainCoordinate, NativeTerrainOffset},
};
use bevy::{math::DVec2, prelude::*};

pub(super) struct SurfaceSegment {
    pub endpoints: [Vec3; 2],
    /// Native plane normal, with positive up component; not a scene direction.
    pub normal: Vec3,
}

/// Native Bevy mesh port: east, absolute interpolated elevation, south metres.
/// No triangle containing a missing source vertex supplies a surface position.
pub(super) fn position(
    terrain: &RegionalTerrain,
    origin: Wgs84CoordinateMicrodegrees,
) -> Option<Vec3> {
    let offset = NativeTerrainCoordinate::from(terrain.request().origin.to_e7())
        .offset_to(origin.to_e7().into());
    position_at_offset(terrain, offset)
}

pub(super) fn position_at_offset(
    terrain: &RegionalTerrain,
    offset: NativeTerrainOffset,
) -> Option<Vec3> {
    let triangle = triangle(terrain, offset)?;
    Some(on_triangle(triangle, offset))
}

/// Endpoints on one grid triangle. The caller splits at grid and diagonal edges.
/// Selecting by the midpoint admits both boundary endpoints to that same plane.
pub(super) fn segment(
    terrain: &RegionalTerrain,
    endpoints: [NativeTerrainOffset; 2],
) -> Option<SurfaceSegment> {
    let [start, end] = endpoints;
    let midpoint = NativeTerrainOffset {
        east_metres: (start.east_metres + end.east_metres) * 0.5,
        north_metres: (start.north_metres + end.north_metres) * 0.5,
    };
    let triangle = triangle(terrain, midpoint)?;
    Some(SurfaceSegment {
        endpoints: endpoints.map(|offset| on_triangle(triangle, offset)),
        normal: (triangle[1] - triangle[0]).cross(triangle[2] - triangle[0]),
    })
}

/// Native homogeneous triangle vertices: X east, Y absolute elevation, Z south.
fn triangle(terrain: &RegionalTerrain, offset: NativeTerrainOffset) -> Option<[Vec3; 3]> {
    let spacing = f64::from(terrain.request().scale.spacing_metres());
    let centre = (REGIONAL_TERRAIN_SIDE - 1) as f64 * 0.5;
    let column = offset.east_metres / spacing + centre;
    let row = offset.north_metres / spacing + centre;
    if !(0.0..=(REGIONAL_TERRAIN_SIDE - 1) as f64).contains(&column)
        || !(0.0..=(REGIONAL_TERRAIN_SIDE - 1) as f64).contains(&row)
    {
        return None;
    }
    let cell_column = column.floor().min((REGIONAL_TERRAIN_SIDE - 2) as f64) as u32;
    let cell_row = row.floor().min((REGIONAL_TERRAIN_SIDE - 2) as f64) as u32;
    let columns = [
        Some(cell_column),
        (column.fract() == 0.0)
            .then_some(cell_column.checked_sub(1))
            .flatten(),
    ];
    let rows = [
        Some(cell_row),
        (row.fract() == 0.0)
            .then_some(cell_row.checked_sub(1))
            .flatten(),
    ];
    let grid = DVec2::new(column, row);
    for row in rows.into_iter().flatten() {
        for column in columns.into_iter().flatten() {
            if let Some(triangle) = triangle_at_cell(terrain, UVec2::new(column, row), grid) {
                return Some(triangle);
            }
        }
    }
    None
}

/// Native mesh grid port: cell and continuous position both use east/north axes.
/// At shared edges either covered triangle is a valid source elevation plane.
fn triangle_at_cell(terrain: &RegionalTerrain, cell: UVec2, grid: DVec2) -> Option<[Vec3; 3]> {
    let fraction = grid - cell.as_dvec2();
    if !fraction.cmpge(DVec2::ZERO).all() || !fraction.cmple(DVec2::ONE).all() {
        return None;
    }
    let south_west = cell.y as usize * REGIONAL_TERRAIN_SIDE + cell.x as usize;
    let south_east = south_west + 1;
    let north_west = south_west + REGIONAL_TERRAIN_SIDE;
    let north_east = north_west + 1;
    let lower = [south_west, south_east, north_west];
    let upper = [south_east, north_east, north_west];
    let triangles = if fraction.x + fraction.y < 1.0 {
        [Some(lower), None]
    } else if fraction.x + fraction.y > 1.0 {
        [Some(upper), None]
    } else {
        [Some(lower), Some(upper)]
    };
    let spacing = f64::from(terrain.request().scale.spacing_metres());
    let centre = (REGIONAL_TERRAIN_SIDE - 1) as f64 * 0.5;
    let vertex = |index: usize| {
        terrain.vertices()[index].map(|vertex| {
            Vec3::new(
                ((index % REGIONAL_TERRAIN_SIDE) as f64 - centre) as f32 * spacing as f32,
                f32::from(vertex.elevation.get()),
                (centre - (index / REGIONAL_TERRAIN_SIDE) as f64) as f32 * spacing as f32,
            )
        })
    };
    for indices in triangles.into_iter().flatten() {
        if let [Some(first), Some(second), Some(third)] = indices.map(vertex) {
            return Some([first, second, third]);
        }
    }
    None
}

fn on_triangle([first, second, third]: [Vec3; 3], offset: NativeTerrainOffset) -> Vec3 {
    let normal = (second - first).cross(third - first);
    let east = offset.east_metres as f32;
    let south = -offset.north_metres as f32;
    let height = first.y - (normal.x * (east - first.x) + normal.z * (south - first.z)) / normal.y;
    Vec3::new(east, height, south)
}
