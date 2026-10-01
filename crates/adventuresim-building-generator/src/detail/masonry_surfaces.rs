//! Structural bearings overlap in volume; each exposed finish has one owner.
use super::*;
use crate::LodVertex;

const PLANE_TOLERANCE_METRES: f32 = 0.0001;
const SAME_NORMAL_MINIMUM: f32 = 0.999_999;
const MINIMUM_FACE_AREA_SQUARE_METRES: f32 = 0.000_001;

struct Plane {
    normal: Vec3,
    origin: Vec3,
    triangles: Vec<SurfaceTriangle>,
}

struct SurfaceTriangle {
    points: [Vec3; 3],
    min: Vec3,
    max: Vec3,
}

impl SurfaceTriangle {
    fn new(points: [Vec3; 3]) -> Self {
        Self {
            points,
            min: points
                .into_iter()
                .fold(Vec3::splat(f32::INFINITY), Vec3::min),
            max: points
                .into_iter()
                .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max),
        }
    }

    fn may_overlap(&self, other: &Self) -> bool {
        // Each face can lie on either side of the shared plane tolerance.
        // Preserve those near-coplanar cuts, including tilted surface normals.
        let margin = Vec3::splat(PLANE_TOLERANCE_METRES * 2.0);
        !self.min.cmpgt(other.max + margin).any() && !other.min.cmpgt(self.max + margin).any()
    }
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
            let triangle = SurfaceTriangle::new(points);
            let mut polygons = vec![points.to_vec()];
            for cut in plane
                .triangles
                .iter()
                .filter(|cut| triangle.may_overlap(cut))
            {
                polygons = polygons
                    .into_iter()
                    .flat_map(|polygon| enclosures::subtract_triangle(polygon, cut.points, normal))
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
            plane.triangles.push(triangle);
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

#[test]
fn bounds_pruning_preserves_disjoint_and_near_coplanar_surface_cuts() {
    let mut pruned = 0;
    for rotation in [
        Quat::IDENTITY,
        Quat::from_rotation_x(0.7),
        Quat::from_rotation_z(0.4),
    ] {
        let normal = rotation * Vec3::Y;
        let points = [Vec3::ZERO, Vec3::X * 2.0, Vec3::Z * 2.0].map(|p| rotation * p);
        let triangle = SurfaceTriangle::new(points);
        for x in -8..=8 {
            for z in -8..=8 {
                for height in [-PLANE_TOLERANCE_METRES, 0.0, PLANE_TOLERANCE_METRES] {
                    let offset = rotation * Vec3::new(x as f32 * 0.5, height, z as f32 * 0.5);
                    let cut = SurfaceTriangle::new(points.map(|p| p + offset));
                    if !triangle.may_overlap(&cut) {
                        assert_eq!(
                            enclosures::subtract_triangle(points.to_vec(), cut.points, normal),
                            vec![points.to_vec()],
                        );
                        pruned += 1;
                    }
                }
            }
        }
    }
    assert!(pruned > 0);
}
