//! Clips semantic city patches to the terrain renderer's own triangle lattice.

use super::*;

const SURFACE_LIFT_METRES: f32 = 0.006;
const SURFACE_PRIORITY_LIFT_METRES: f32 = 0.001;
const MARKET_PRIORITY_LIFT_METRES: f32 = 0.002;
const YARD_SURFACE_LIFT_METRES: f32 = 0.003;

#[derive(Default)]
pub(super) struct CitySurfaceMeshBuilder {
    chunks: std::collections::BTreeMap<traffic::TrafficTile, SurfaceVertices>,
}

#[derive(Default)]
struct SurfaceVertices {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    footprints: Vec<[f32; 4]>,
    activities: Vec<[f32; 2]>,
}

struct SurfacePatch {
    corners: [Vec2; 4],
    lift_metres: f32,
    kind: PatchKind,
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum PatchKind {
    Corridor,
    Market,
    Yard,
}

impl CitySurfaceMeshBuilder {
    pub(super) fn append_street(
        &mut self,
        street: CityStreetPatch,
        support: &GroundSupport,
        groups: &[FurnitureGroup],
    ) {
        let lift_metres = SURFACE_LIFT_METRES
            + f32::from(street.surface().priority()) * SURFACE_PRIORITY_LIFT_METRES;
        let patch = match street {
            CityStreetPatch::Corridor {
                start_metres,
                end_metres,
                half_width_metres,
                ..
            } => {
                let tangent = (end_metres - start_metres).normalize();
                let normal = Vec2::new(-tangent.y, tangent.x) * half_width_metres;
                SurfacePatch {
                    corners: [
                        start_metres - normal,
                        start_metres + normal,
                        end_metres + normal,
                        end_metres - normal,
                    ],
                    lift_metres,
                    kind: PatchKind::Corridor,
                }
            }
            CityStreetPatch::Market { corners_metres, .. } => SurfacePatch {
                corners: corners_metres,
                // The plaza owns crossing wear above the streets beneath it.
                lift_metres: lift_metres + MARKET_PRIORITY_LIFT_METRES,
                kind: PatchKind::Market,
            },
        };
        self.append(patch, support, groups);
    }

    pub(super) fn append_yard(
        &mut self,
        yard: CityYardPatch,
        support: &GroundSupport,
        groups: &[FurnitureGroup],
    ) {
        self.append(
            SurfacePatch {
                corners: yard.corners_metres,
                lift_metres: YARD_SURFACE_LIFT_METRES,
                kind: PatchKind::Yard,
            },
            support,
            groups,
        );
    }

    fn append(&mut self, patch: SurfacePatch, support: &GroundSupport, groups: &[FurnitureGroup]) {
        let [a, b, c, d] = patch.corners;
        let width = a.distance(b).max(d.distance(c));
        let depth = a.distance(d).max(b.distance(c));
        let activity = activity::ActivityWear::for_patch(patch.corners, groups);
        support.clip(patch.corners, |triangle| {
            let normal = (triangle[1] - triangle[0])
                .cross(triangle[2] - triangle[0])
                .normalize();
            let minimum = triangle
                .map(|p| p.xz())
                .into_iter()
                .fold(Vec2::splat(f32::INFINITY), Vec2::min);
            let maximum = triangle
                .map(|p| p.xz())
                .into_iter()
                .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
            for tile in traffic::TrafficTile::covering(minimum, maximum) {
                let polygon = support::clip_polygon(triangle.to_vec(), tile.corners());
                for index in 1..polygon.len().saturating_sub(1) {
                    let clipped = [polygon[0], polygon[index], polygon[index + 1]];
                    if (clipped[1] - clipped[0])
                        .cross(clipped[2] - clipped[0])
                        .length_squared()
                        <= f32::EPSILON
                    {
                        continue;
                    }
                    let vertices = self.chunks.entry(tile).or_default();
                    for mut point in clipped {
                        let uv = support::footprint_uv(patch.corners, point.xz());
                        vertices.uvs.push(uv.to_array());
                        vertices.activities.push(activity.at(point.xz()).to_array());
                        vertices
                            .footprints
                            .push([width, depth, f32::from(patch.kind as u8), 1.0]);
                        point.y += patch.lift_metres;
                        vertices.positions.push(point.to_array());
                        vertices.normals.push(normal.to_array());
                    }
                }
            }
        });
    }

    pub(super) fn build(self) -> impl Iterator<Item = (traffic::TrafficTile, Mesh)> {
        self.chunks
            .into_iter()
            .map(|(tile, vertices)| (tile, vertices.build()))
    }
}

impl SurfaceVertices {
    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, self.activities);
        // RGB encodes physical patch dimensions and shape, not a vertex tint.
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.footprints);
        mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipped_road_clears_spikes_and_true_terrain_diagonals_at_triangle_interiors() {
        let terrain = SceneTerrain::new(12, 12, 1.0, |point| {
            if point.x == 6.0 && point.y == 6.0 {
                0.4
            } else {
                point.x * 0.013 + point.y * 0.007
            }
        });
        let mut support = GroundSupport::default();
        support.add_mesh(&terrain.mesh(), Vec3::ZERO);
        for reverse in [false, true] {
            let mut corners = [
                Vec2::new(-4.1, -3.2),
                Vec2::new(3.3, -4.0),
                Vec2::new(4.2, 3.7),
                Vec2::new(-3.8, 4.1),
            ];
            if reverse {
                corners.reverse();
            }
            let mut builder = CitySurfaceMeshBuilder::default();
            builder.append_street(
                CityStreetPatch::Market {
                    corners_metres: corners,
                    surface: CityStreetSurface::Fieldstone,
                },
                &support,
                &[],
            );
            assert!(!builder.chunks.is_empty());
            for vertices in builder.chunks.values() {
                for points in vertices.positions.as_chunks::<3>().0 {
                    let [a, b, c] = [points[0], points[1], points[2]].map(Vec3::from_array);
                    assert!((b - a).cross(c - a).y > 0.0);
                    // Several interior barycentric probes catch missing spikes
                    // and the wrong diagonal, rather than just testing vertices.
                    for weights in [
                        Vec3::splat(1.0 / 3.0),
                        Vec3::new(0.1, 0.3, 0.6),
                        Vec3::new(0.6, 0.1, 0.3),
                    ] {
                        let point = a * weights.x + b * weights.y + c * weights.z;
                        let clearance = point.y - terrain.height_at(point.xz()).unwrap();
                        assert!(
                            (clearance - 0.01).abs() < 0.0001,
                            "clearance {clearance} at {point:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn absent_source_terrain_cannot_create_default_height_geometry() {
        let terrain = SceneTerrain::new(8, 8, 1.0, |_| 7.0);
        let mut support = GroundSupport::default();
        support.add_mesh(&terrain.mesh(), Vec3::ZERO);
        let mut builder = CitySurfaceMeshBuilder::default();
        builder.append_street(
            CityStreetPatch::Corridor {
                start_metres: Vec2::new(-12.0, 0.0),
                end_metres: Vec2::new(12.0, 0.0),
                half_width_metres: 2.0,
                surface: CityStreetSurface::Fieldstone,
            },
            &support,
            &[],
        );
        assert!(!builder.chunks.is_empty());
        assert!(
            builder
                .chunks
                .values()
                .flat_map(|v| &v.positions)
                .all(|point| point[1] > 7.0 && point[0].abs() <= 4.0001)
        );
    }
}
