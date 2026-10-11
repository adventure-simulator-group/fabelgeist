//! Clip geographic ribbons to the same graded triangles as city paving.
use super::*;
use crate::presentation::regional_map::city::CitySurface;

impl PathMeshBuilder {
    pub(super) fn add_city(
        &mut self,
        terrain: &RegionalTerrain,
        line: GeographicLine<'_>,
        width: f32,
        city: CitySurface<'_>,
    ) -> Result<()> {
        let window = NativeTerrainCoordinate::from(terrain.request().origin.to_e7());
        let spacing = f64::from(terrain.request().scale.spacing_metres());
        let half = (REGIONAL_TERRAIN_SIDE - 1) as f64 * 0.5;
        let world_from_city = city.frame.world_from_city();
        let city_from_world = world_from_city.inverse();
        for points in line.segments() {
            let grid = points.map(|point| {
                let offset = window.offset_to(point.into());
                DVec2::new(
                    offset.east_metres / spacing + half,
                    offset.north_metres / spacing + half,
                )
            });
            let Some([start, end]) = clip(grid, (REGIONAL_TERRAIN_SIDE - 1) as f64) else {
                continue;
            };
            let length = (end - start).length();
            let intervals = if line.dashed() {
                (length / ESTIMATE_DASH_CELLS).ceil() as usize
            } else {
                1
            };
            for interval in 0..intervals {
                if line.dashed() && !interval.is_multiple_of(2) {
                    continue;
                }
                let endpoints = [interval, interval + 1].map(|index| {
                    let point = start.lerp(end, index as f64 / intervals as f64);
                    Vec3::new(
                        ((point.x - half) * spacing) as f32,
                        0.0,
                        -((point.y - half) * spacing) as f32,
                    )
                });
                let [start, end] = endpoints;
                let Some(direction) = (end - start).try_normalize() else {
                    continue;
                };
                let side = direction.cross(Vec3::Y) * width * 0.5;
                let corners = [start - side, start + side, end + side, end - side]
                    .map(|point| city_from_world.transform_point3(point).xz());
                // The source projection above and CityFrame both use the shared
                // sampler scale. This handoff carries canonical city metres.
                let mut exceeded = false;
                city.support.clip(corners, |triangle| {
                    if self.positions.len() + 3 > MAX_PATH_MESH_VERTICES {
                        exceeded = true;
                        return;
                    }
                    let first = self.positions.len() as u32;
                    self.positions.extend(triangle.map(|point| {
                        (world_from_city.transform_point3(point.as_vec3())
                            + Vec3::Y * line.clearance_metres())
                        .to_array()
                    }));
                    // City north becomes world south: reverse source winding.
                    self.indices.extend([first, first + 2, first + 1]);
                });
                if exceeded {
                    return Err(PathGeometryError::CapacityExceeded);
                }
            }
        }
        Ok(())
    }
}
