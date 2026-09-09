//! Street clipping against the actual terrain triangles, including vista seams.

use super::*;
use bevy::mesh::VertexAttributeValues;
use std::collections::BTreeMap;

const SUPPORT_CHUNK_METRES: f32 = 32.0;
const CLIP_EPSILON: f32 = 0.00001;

#[derive(Clone, Default)]
pub(in crate::presentation::vista) struct GroundSupport {
    chunks: BTreeMap<(i32, i32), SupportChunk>,
}

#[derive(Clone)]
struct SupportChunk {
    minimum: Vec2,
    maximum: Vec2,
    triangles: Vec<[Vec3; 3]>,
}

impl GroundSupport {
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
        for indices in indices.as_chunks::<3>().0 {
            let triangle = [indices[0], indices[1], indices[2]]
                .map(|index| Vec3::from_array(positions[index]) + origin);
            // Vista skirts and cliff walls are not walkable ground support.
            if (triangle[1] - triangle[0])
                .cross(triangle[2] - triangle[0])
                .y
                <= CLIP_EPSILON
            {
                continue;
            }
            let centre = (triangle[0] + triangle[1] + triangle[2]) / 3.0;
            let key = (centre.xz() / SUPPORT_CHUNK_METRES).floor().as_ivec2();
            let chunk = self
                .chunks
                .entry((key.x, key.y))
                .or_insert_with(|| SupportChunk {
                    minimum: Vec2::splat(f32::INFINITY),
                    maximum: Vec2::splat(f32::NEG_INFINITY),
                    triangles: Vec::new(),
                });
            for point in triangle {
                chunk.minimum = chunk.minimum.min(point.xz());
                chunk.maximum = chunk.maximum.max(point.xz());
            }
            chunk.triangles.push(triangle);
        }
    }

    pub(super) fn clip(&self, corners: [Vec2; 4], mut emit: impl FnMut([Vec3; 3])) {
        let minimum = corners
            .into_iter()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let maximum = corners
            .into_iter()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        for chunk in self.chunks.values().filter(|chunk| {
            chunk.maximum.cmpge(minimum).all() && chunk.minimum.cmple(maximum).all()
        }) {
            for &triangle in &chunk.triangles {
                let tri_minimum = triangle
                    .map(|point| point.xz())
                    .into_iter()
                    .fold(Vec2::splat(f32::INFINITY), Vec2::min);
                let tri_maximum = triangle
                    .map(|point| point.xz())
                    .into_iter()
                    .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
                if !tri_maximum.cmpge(minimum).all() || !tri_minimum.cmple(maximum).all() {
                    continue;
                }
                let polygon = clip_polygon(triangle.to_vec(), corners);
                for index in 1..polygon.len().saturating_sub(1) {
                    let clipped = [polygon[0], polygon[index], polygon[index + 1]];
                    if (clipped[1] - clipped[0])
                        .cross(clipped[2] - clipped[0])
                        .length_squared()
                        > CLIP_EPSILON * CLIP_EPSILON
                    {
                        emit(clipped);
                    }
                }
            }
        }
    }
}

fn clip_polygon(mut polygon: Vec<Vec3>, corners: [Vec2; 4]) -> Vec<Vec3> {
    let winding = (corners[1] - corners[0])
        .perp_dot(corners[3] - corners[0])
        .signum();
    for side in 0..4 {
        let start = corners[side];
        let edge = corners[(side + 1) % 4] - start;
        let distance = |point: Vec3| edge.perp_dot(point.xz() - start) * winding;
        let mut clipped = Vec::new();
        for index in 0..polygon.len() {
            let a = polygon[index];
            let b = polygon[(index + 1) % polygon.len()];
            let da = distance(a);
            let db = distance(b);
            if da >= -CLIP_EPSILON {
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
