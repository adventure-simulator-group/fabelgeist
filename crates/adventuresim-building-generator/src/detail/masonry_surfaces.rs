//! Structural bearings overlap in volume; each exposed finish has one owner.
use super::*;
use crate::LodVertex;

const PLANE_TOLERANCE_METRES: f32 = 0.0001;
const SAME_NORMAL_MINIMUM: f32 = 0.999_999;
const MINIMUM_FACE_AREA_SQUARE_METRES: f32 = 0.000_001;

struct Plane {
    normal: Vec3,
    origin: Vec3,
    triangles: Vec<[Vec3; 3]>,
}

/// Union coplanar finishes without changing the canonical load-bearing solids.
/// Earlier faces retain their material and UVs, including lintel end bearings.
pub(crate) fn resolve(detail: &mut BuildingDetail) {
    let mut planes: Vec<Plane> = Vec::new();
    for mesh in detail.meshes.iter_mut().filter(|mesh| {
        matches!(
            mesh.material,
            BuildingLodMaterial::Wall(_)
                | BuildingLodMaterial::InteriorPlaster
                | BuildingLodMaterial::DressedStone
        )
    }) {
        let source = std::mem::replace(mesh, LodMesh::new(mesh.material));
        for indices in source.indices.as_chunks::<3>().0 {
            let vertices = indices.map(|i| source.vertices[i as usize]);
            let points = vertices.map(|v| v.position);
            let normal = vertices[0].normal;
            let plane_index = planes
                .iter()
                .position(|plane| {
                    plane.normal.dot(normal) >= SAME_NORMAL_MINIMUM
                        && points.iter().all(|point| {
                            (*point - plane.origin).dot(plane.normal).abs()
                                <= PLANE_TOLERANCE_METRES
                        })
                })
                .unwrap_or_else(|| {
                    planes.push(Plane {
                        normal,
                        origin: points[0],
                        triangles: Vec::new(),
                    });
                    planes.len() - 1
                });
            let plane = &mut planes[plane_index];
            let mut polygons = vec![points.to_vec()];
            for &cut in &plane.triangles {
                polygons = polygons
                    .into_iter()
                    .flat_map(|polygon| enclosures::subtract_triangle(polygon, cut, normal))
                    .collect();
                if polygons.is_empty() {
                    break;
                }
            }
            for polygon in polygons {
                for index in 1..polygon.len().saturating_sub(1) {
                    let piece = [polygon[0], polygon[index], polygon[index + 1]];
                    if (piece[1] - piece[0]).cross(piece[2] - piece[0]).length() * 0.5
                        > MINIMUM_FACE_AREA_SQUARE_METRES
                    {
                        mesh.push_triangle(
                            piece,
                            normal,
                            piece.map(|p| interpolate_uv(vertices, p)),
                        );
                    }
                }
            }
            // The original triangle covers exactly the union of its surviving
            // pieces and prior faces, avoiding fragmentation in later cuts.
            plane.triangles.push(points);
        }
    }
}

fn interpolate_uv(vertices: [LodVertex; 3], point: Vec3) -> Vec2 {
    let a = vertices[1].position - vertices[0].position;
    let b = vertices[2].position - vertices[0].position;
    let p = point - vertices[0].position;
    let normal = a.cross(b);
    let denominator = normal.length_squared();
    let u = p.cross(b).dot(normal) / denominator;
    let v = a.cross(p).dot(normal) / denominator;
    vertices[0].uv + (vertices[1].uv - vertices[0].uv) * u + (vertices[2].uv - vertices[0].uv) * v
}
