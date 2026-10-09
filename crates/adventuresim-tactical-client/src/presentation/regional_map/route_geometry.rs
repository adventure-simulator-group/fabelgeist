//! Clip a geographic route to covered source triangles before emitting ribbons.
use super::geographic_surface;
use adventuresim_tactical_core::{
    regional_map::{MapRoute, MapRouteKind},
    regional_terrain::{REGIONAL_TERRAIN_SIDE, RegionalTerrain},
};
use adventuresim_world_schema::coordinates::terrain_projection::{
    NativeTerrainCoordinate, NativeTerrainOffset,
};
use bevy::{
    asset::RenderAssetUsages,
    math::DVec2,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

const ROUTE_SURFACE_CLEARANCE_METRES: f32 = 0.25;
const ESTIMATE_DASH_CELLS: f64 = 0.5;
const MAX_ROUTE_MESH_VERTICES: usize = 262_144;
type Result<T> = std::result::Result<T, RouteGeometryError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum RouteGeometryError {
    #[error("map route exceeds {MAX_ROUTE_MESH_VERTICES} GPU vertices")]
    CapacityExceeded,
}

/// Width enters the native mesh adapter in metres, derived from physical pixels.
pub(super) fn mesh(
    terrain: &RegionalTerrain,
    route: &MapRoute,
    width_metres: f32,
) -> Result<Option<Mesh>> {
    let origin = NativeTerrainCoordinate::from(terrain.request().origin.to_e7());
    let spacing = f64::from(terrain.request().scale.spacing_metres());
    let half = (REGIONAL_TERRAIN_SIDE - 1) as f64 * 0.5;
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for points in route.points().windows(2) {
        let native = [points[0], points[1]].map(|point| origin.offset_to(point.to_e7()));
        let grid = native.map(|offset| {
            DVec2::new(
                offset.east_metres / spacing + half,
                offset.north_metres / spacing + half,
            )
        });
        let Some([start, end]) = clip(grid, (REGIONAL_TERRAIN_SIDE - 1) as f64) else {
            continue;
        };
        let mut knots = vec![0.0, 1.0];
        for axis in [DVec2::X, DVec2::Y, DVec2::ONE] {
            let first = start.dot(axis);
            let last = end.dot(axis);
            if first == last {
                continue;
            }
            for edge in first.min(last).ceil() as i32..=first.max(last).floor() as i32 {
                let parameter = (f64::from(edge) - first) / (last - first);
                if parameter > 0.0 && parameter < 1.0 {
                    knots.push(parameter);
                }
            }
        }
        // The estimate has visible gaps; computed routes remain continuous.
        if route.kind() == MapRouteKind::Estimate {
            let count = ((end - start).length() / ESTIMATE_DASH_CELLS).ceil() as usize;
            for index in 1..count {
                knots.push(index as f64 / count as f64);
            }
        }
        knots.sort_by(f64::total_cmp);
        knots.dedup();
        for bounds in knots.windows(2) {
            let midpoint = start.lerp(end, (bounds[0] + bounds[1]) * 0.5);
            if route.kind() == MapRouteKind::Estimate
                && !(((midpoint - start).length() / ESTIMATE_DASH_CELLS).floor() as usize)
                    .is_multiple_of(2)
            {
                continue;
            }
            let endpoints = [bounds[0], bounds[1]].map(|parameter| {
                let grid = start.lerp(end, parameter);
                NativeTerrainOffset {
                    east_metres: (grid.x - half) * spacing,
                    north_metres: (grid.y - half) * spacing,
                }
            });
            if let Some(segment) = geographic_surface::segment(terrain, endpoints) {
                ribbon(segment, width_metres, &mut positions, &mut indices);
                if positions.len() > MAX_ROUTE_MESH_VERTICES {
                    return Err(RouteGeometryError::CapacityExceeded);
                }
            }
        }
    }
    if indices.is_empty() {
        return Ok(None);
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    Ok(Some(mesh))
}

/// Native 2D grid kernel: restrict a homogeneous endpoint pair to the square.
fn clip([start, end]: [DVec2; 2], maximum: f64) -> Option<[DVec2; 2]> {
    let direction = end - start;
    let mut enter: f64 = 0.0;
    let mut leave: f64 = 1.0;
    for axis in [DVec2::X, DVec2::Y] {
        let origin = start.dot(axis);
        let delta = direction.dot(axis);
        if delta == 0.0 {
            if !(0.0..=maximum).contains(&origin) {
                return None;
            }
        } else {
            let first = -origin / delta;
            let last = (maximum - origin) / delta;
            enter = enter.max(first.min(last));
            leave = leave.min(first.max(last));
            if enter >= leave {
                return None;
            }
        }
    }
    Some([start + direction * enter, start + direction * leave])
}

fn ribbon(
    segment: geographic_surface::SurfaceSegment,
    width: f32,
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
) {
    let [start, end] = segment.endpoints;
    let direction = Vec3::new(end.x - start.x, 0.0, end.z - start.z);
    let Some(tangent) = direction.try_normalize() else {
        return;
    };
    let mut side = tangent.cross(Vec3::Y) * width * 0.5;
    side.y = -(segment.normal.x * side.x + segment.normal.z * side.z) / segment.normal.y;
    let clearance = Vec3::Y * ROUTE_SURFACE_CLEARANCE_METRES;
    let first = positions.len() as u32;
    positions.extend(
        [
            start - side + clearance,
            start + side + clearance,
            end - side + clearance,
            end + side + clearance,
        ]
        .map(|point| point.to_array()),
    );
    // Upward-facing triangles in the native east/up/south mesh frame.
    indices.extend([first, first + 1, first + 2, first + 1, first + 3, first + 2]);
}
