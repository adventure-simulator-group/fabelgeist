//! Street clipping against the actual terrain triangles, including vista seams.

use super::*;
use adventuresim_tactical_core::scene_coordinates::ScenePlanPoint;
use bevy::mesh::VertexAttributeValues;
mod index;

const CLIP_EPSILON: f32 = 0.00001;

#[derive(Clone, Default)]
pub(in crate::presentation) struct GroundSupport {
    triangles: Vec<[Vec3; 3]>,
    index: std::sync::OnceLock<index::TriangleIndex>,
}

impl GroundSupport {
    /// Highest point on the presented city ground, in canonical east/up/north
    /// metres. Missing coverage never substitutes a heightfield or nearest face.
    pub(in crate::presentation) fn position(&self, point: ScenePlanPoint) -> Option<Vec3> {
        let point = point.metres();
        let index = self
            .index
            .get_or_init(|| index::TriangleIndex::new(&self.triangles));
        let mut highest: Option<Vec3> = None;
        index.query(point, point, |index| {
            let triangle = self.triangles[index].map(Vec3::as_dvec3);
            let query = point.as_dvec2();
            let [a, b, c] = triangle;
            // Upward-facing source triangles are clockwise in the east/north
            // plan. Classify edges in double precision, as paving clipping does.
            if [(a, b), (b, c), (c, a)].into_iter().any(|(start, end)| {
                (end.xz() - start.xz()).perp_dot(query - start.xz()) > f64::from(CLIP_EPSILON)
            }) {
                return;
            }
            let normal = (b - a).cross(c - a);
            let height = a.y - (normal.x * (query.x - a.x) + normal.z * (query.y - a.z)) / normal.y;
            let position = Vec3::new(point.x, height as f32, point.y);
            if highest.is_none_or(|current| position.y > current.y) {
                highest = Some(position);
            }
        });
        highest
    }

    pub(in crate::presentation::vista) fn add_mesh(&mut self, mesh: &Mesh, origin: Vec3) {
        let Some(positions) = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
        else {
            return;
        };
        let Some(indices) = mesh.indices() else {
            return;
        };
        let indices = indices.iter().collect::<Vec<_>>();
        let triangles = indices.as_chunks::<3>().0.iter().map(|indices| {
            [indices[0], indices[1], indices[2]].map(|index| Vec3::from_array(positions[index]))
        });
        self.add_triangles(triangles, origin);
    }

    /// Owned terrain already carries the exact triangles. Avoid materializing
    /// renderer vertices, normals, UVs and indices just to discard buried and
    /// vertical faces for street clipping. The eligibility rule is shared.
    pub(in crate::presentation::vista) fn add_triangles(
        &mut self,
        triangles: impl IntoIterator<Item = [Vec3; 3]>,
        origin: Vec3,
    ) {
        for triangle in triangles {
            let triangle = triangle.map(|point| point + origin);
            // Vista skirts and cliff walls are not walkable ground support.
            if (triangle[1] - triangle[0])
                .cross(triangle[2] - triangle[0])
                .y
                <= CLIP_EPSILON
            {
                continue;
            }
            self.triangles.push(triangle);
        }
        self.index.take();
    }

    pub(in crate::presentation) fn clip(
        &self,
        corners: [Vec2; 4],
        mut emit: impl FnMut([bevy::math::DVec3; 3]),
    ) {
        let minimum = corners
            .into_iter()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let maximum = corners
            .into_iter()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        let index = self
            .index
            .get_or_init(|| index::TriangleIndex::new(&self.triangles));
        index.query(minimum, maximum, |index| {
            let triangle = self.triangles[index];
            let tri_minimum = triangle
                .map(|point| point.xz())
                .into_iter()
                .fold(Vec2::splat(f32::INFINITY), Vec2::min);
            let tri_maximum = triangle
                .map(|point| point.xz())
                .into_iter()
                .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
            if !tri_maximum.cmpge(minimum).all() || !tri_minimum.cmple(maximum).all() {
                return;
            }
            let polygon = clip_polygon(triangle.map(Vec3::as_dvec3).to_vec(), corners);
            for index in 1..polygon.len().saturating_sub(1) {
                let clipped = [polygon[0], polygon[index], polygon[index + 1]];
                if (clipped[1] - clipped[0])
                    .cross(clipped[2] - clipped[0])
                    .length_squared()
                    > f64::from(CLIP_EPSILON * CLIP_EPSILON)
                {
                    emit(clipped);
                }
            }
        });
    }
}

pub(super) fn clip_polygon(
    mut polygon: Vec<bevy::math::DVec3>,
    corners: [Vec2; 4],
) -> Vec<bevy::math::DVec3> {
    // Keep intersections in double precision until the represented polygon is
    // complete. Repeated f32 lerps can move a constant source edge by one ULP
    // across a retaining boundary and place paving on the wrong ground level.
    let corners = corners.map(Vec2::as_dvec2);
    let winding = (corners[1] - corners[0])
        .perp_dot(corners[3] - corners[0])
        .signum();
    for side in 0..4 {
        let start = corners[side];
        let edge = corners[(side + 1) % 4] - start;
        let distance = |point: bevy::math::DVec3| edge.perp_dot(point.xz() - start) * winding;
        let mut clipped = Vec::new();
        for index in 0..polygon.len() {
            let a = polygon[index];
            let b = polygon[(index + 1) % polygon.len()];
            let da = distance(a);
            let db = distance(b);
            if da >= -f64::from(CLIP_EPSILON) {
                clipped.push(a);
            }
            if (da >= 0.0) != (db >= 0.0) {
                clipped.push(a.lerp(b, da / (da - db)));
            }
        }
        polygon = clipped;
        if polygon.is_empty() {
            break;
        }
    }
    polygon
}

/// Inverse bilinear coordinates preserve the semantic market/yard footprint
/// when terrain clipping inserts vertices on an irregular block boundary.
pub(super) fn footprint_uv(corners: [Vec2; 4], point: Vec2) -> Vec2 {
    let [a, b, c, d] = corners;
    let mut uv = Vec2::splat(0.5);
    for _ in 0..6 {
        let residual = a.lerp(b, uv.x).lerp(d.lerp(c, uv.x), uv.y) - point;
        let du = (b - a).lerp(c - d, uv.y);
        let dv = (d - a).lerp(c - b, uv.x);
        let determinant = du.perp_dot(dv);
        if determinant.abs() < CLIP_EPSILON {
            break;
        }
        uv -= Vec2::new(residual.perp_dot(dv), du.perp_dot(residual)) / determinant;
    }
    uv.clamp(Vec2::ZERO, Vec2::ONE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_owned_faces_clip_exactly_like_meshes_without_buried_or_vertical_support() {
        let faces = [
            [
                Vec3::new(-2.0, 1.0, -2.0),
                Vec3::new(-2.0, 1.0, 2.0),
                Vec3::new(2.0, 2.0, -2.0),
            ],
            [
                Vec3::new(-2.0, -1.0, -2.0),
                Vec3::new(2.0, -1.0, -2.0),
                Vec3::new(-2.0, -1.0, 2.0),
            ],
            [
                Vec3::new(-2.0, -1.0, -2.0),
                Vec3::new(-2.0, 1.0, -2.0),
                Vec3::new(2.0, 1.0, -2.0),
            ],
        ];
        let positions: Vec<_> = faces.iter().flatten().map(|p| p.to_array()).collect();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_indices(Indices::U32((0..positions.len() as u32).collect()));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        let origin = Vec3::new(0.5, 3.0, -0.25);
        let mut from_mesh = GroundSupport::default();
        from_mesh.add_mesh(&mesh, origin);
        let mut direct = GroundSupport::default();
        direct.add_triangles(faces, origin);
        let corners = [
            Vec2::splat(-0.5),
            Vec2::new(0.5, -0.5),
            Vec2::splat(0.5),
            Vec2::new(-0.5, 0.5),
        ];
        let mut expected = Vec::new();
        let mut actual = Vec::new();
        from_mesh.clip(corners, |face| expected.push(face));
        direct.clip(corners, |face| actual.push(face));
        assert!(!actual.is_empty());
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&actual),
            bytemuck::cast_slice::<_, u8>(&expected)
        );
        assert!(actual.iter().flatten().all(|point| point.y >= 4.0));
    }

    #[test]
    fn coplanar_subdivision_preserves_clipped_area_and_separate_retaining_levels() {
        for yaw in [0.0, 0.73] {
            let rotation = Quat::from_rotation_y(yaw);
            let origin = Vec3::new(37.5, 5.0, -81.25);
            let world = |p: Vec3| rotation * p + origin;
            let build = |divisions: usize| {
                let mut support = GroundSupport::default();
                for (start, floor) in [(-4.0, 3.0), (0.0, 0.0)] {
                    let point = |x: usize, z: usize| {
                        let x = start + 4.0 * x as f32 / divisions as f32;
                        let z = -3.0 + 6.0 * z as f32 / divisions as f32;
                        world(Vec3::new(x, floor + 0.15 * x + 0.2 * z, z))
                    };
                    for x in 0..divisions {
                        for z in 0..divisions {
                            let [a, b, c, d] = [
                                point(x, z),
                                point(x + 1, z),
                                point(x + 1, z + 1),
                                point(x, z + 1),
                            ];
                            support.add_triangles([[a, c, b], [a, d, c]], Vec3::ZERO);
                        }
                    }
                }
                support
            };
            let corners = [
                Vec2::new(-3.0, -2.0),
                Vec2::new(3.0, -2.0),
                Vec2::new(3.0, 2.0),
                Vec2::new(-3.0, 2.0),
            ]
            .map(|p| world(Vec3::new(p.x, 0.0, p.y)).xz());
            let mut coarse = Vec::new();
            let mut fine = Vec::new();
            build(1).clip(corners, |face| coarse.push(face.map(|p| p.as_vec3())));
            build(8).clip(corners, |face| fine.push(face.map(|p| p.as_vec3())));
            assert!(coarse.len() < fine.len());
            for faces in [&coarse, &fine] {
                let area: f64 = faces
                    .iter()
                    .map(|[a, b, c]| {
                        (b.xz().as_dvec2() - a.xz().as_dvec2())
                            .perp_dot(c.xz().as_dvec2() - a.xz().as_dvec2())
                            .abs()
                            * 0.5
                    })
                    .sum();
                assert!((area - 24.0).abs() < 0.001, "clipped area {area}");
                for face in faces {
                    let local =
                        rotation.inverse() * (face.iter().copied().sum::<Vec3>() / 3.0 - origin);
                    let floor = if local.x < 0.0 { 3.0 } else { 0.0 };
                    let expected = floor + 0.15 * local.x + 0.2 * local.z;
                    assert!(
                        (local.y - expected).abs() < 0.001,
                        "clipping changes the support at {local:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn traffic_partition_preserves_the_constant_goslar_retaining_edge() {
        let triangle = [
            Vec3::new(-215.999, 3.5755415, 228.77707),
            Vec3::new(-212.54999, 3.5051064, 228.25533),
            Vec3::new(-215.999, 3.5651064, 228.25533),
        ];
        let corners = [
            Vec2::new(-216.0, 228.0),
            Vec2::new(-214.0, 228.0),
            Vec2::new(-214.0, 230.0),
            Vec2::new(-216.0, 230.0),
        ];
        let polygon = clip_polygon(triangle.map(Vec3::as_dvec3).to_vec(), corners);
        assert!(polygon.len() >= 3);
        assert!(
            polygon
                .iter()
                .all(|point| point.z >= f64::from(triangle[1].z))
        );
        let on_edge = polygon
            .iter()
            .find(|point| point.x == -214.0 && point.z == f64::from(triangle[1].z))
            .expect("traffic boundary retains the exact source-edge coordinate");
        assert!((on_edge.y - 3.530331).abs() < 0.000001);
    }

    #[test]
    fn triangle_reaching_across_a_chunk_boundary_remains_queryable() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[-1.0, 0.0, -2.0], [-1.0, 0.0, 2.0], [100.0, 0.0, -2.0]],
        );
        mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
        let mut support = GroundSupport::default();
        support.add_mesh(&mesh, Vec3::ZERO);
        let mut triangles = Vec::new();

        support.clip(
            [
                Vec2::new(-0.5, -0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(0.5, 0.5),
                Vec2::new(-0.5, 0.5),
            ],
            |triangle| triangles.push(triangle),
        );

        assert!(!triangles.is_empty());
    }

    #[test]
    fn retained_vista_triangle_diagonal_is_preserved_and_vertical_skirts_are_excluded() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [-2.0, 0.0, -2.0],
                [2.0, 0.0, -2.0],
                [2.0, 1.0, 2.0],
                [-2.0, 0.0, 2.0],
                [-2.0, -10.0, -2.0],
                [2.0, -10.0, -2.0],
            ],
        );
        mesh.insert_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2, 0, 1, 4, 1, 5, 4]));
        let mut support = GroundSupport::default();
        support.add_mesh(&mesh, Vec3::ZERO);
        let mut count = 0;
        support.clip(
            [
                Vec2::new(-1.0, -1.0),
                Vec2::new(1.0, -1.0),
                Vec2::ONE,
                Vec2::new(-1.0, 1.0),
            ],
            |triangle| {
                let centre = (triangle[0] + triangle[1] + triangle[2]) / 3.0;
                let expected = ((centre.x + 2.0) / 4.0).min((centre.z + 2.0) / 4.0);
                assert!((centre.y - expected).abs() < 0.0001);
                assert!(triangle.into_iter().all(|point| point.y >= 0.0));
                count += 1;
            },
        );
        assert!(count >= 2);
    }
}
