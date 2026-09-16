//! Finish ownership and hidden coplanar seams at wall-to-gable junctions.
use super::*;

const SURFACE_PLANE_TOLERANCE_METRES: f32 = 0.0001;
const SAME_NORMAL_MINIMUM: f32 = 0.999_999;
const MINIMUM_SEAM_AREA_SQUARE_METRES: f32 = 0.000_001;

pub(super) fn append(
    detail: &mut BuildingDetail,
    plan: &BuildingPlan,
    enclosure: &crate::RoofEnclosureFace,
) {
    for triangle in tessellate_roof_enclosure(enclosure, &plan.wall_assemblies) {
        let material = material(plan, enclosure.material, triangle.surface);
        let mut polygons = vec![triangle.positions.to_vec()];
        // Wall faces own their exposed end returns. A cheek reaching that return
        // contributes only the remaining surface, avoiding two co-facing finishes.
        for mesh in detail
            .meshes
            .iter()
            .filter(|mesh| matches!(mesh.material, BuildingLodMaterial::Wall(_)))
        {
            for indices in mesh.indices.as_chunks::<3>().0 {
                let vertices = indices.map(|i| mesh.vertices[i as usize]);
                if vertices[0].normal.dot(triangle.normal) < SAME_NORMAL_MINIMUM
                    || vertices.iter().any(|v| {
                        ((v.position - triangle.positions[0]).dot(triangle.normal)).abs()
                            > SURFACE_PLANE_TOLERANCE_METRES
                    })
                {
                    continue;
                }
                polygons = polygons
                    .into_iter()
                    .flat_map(|polygon| {
                        subtract_triangle(polygon, vertices.map(|v| v.position), triangle.normal)
                    })
                    .collect();
            }
        }
        let mesh = detail.mesh_mut(material);
        for polygon in polygons {
            for index in 1..polygon.len().saturating_sub(1) {
                let positions = [polygon[0], polygon[index], polygon[index + 1]];
                let piece = crate::RoofSurfaceTriangle {
                    positions,
                    ..triangle
                };
                if (positions[1] - positions[0])
                    .cross(positions[2] - positions[0])
                    .length()
                    * 0.5
                    > MINIMUM_SEAM_AREA_SQUARE_METRES
                {
                    mesh.push_triangle(
                        positions,
                        triangle.normal,
                        piece.planar_uvs(BUILDING_DETAIL_UV_METRES_PER_UNIT),
                    );
                }
            }
        }
    }
}

pub(super) fn material(
    plan: &BuildingPlan,
    exterior: RoofMaterial,
    surface: RoofSurface,
) -> BuildingLodMaterial {
    let boarded_belfry = plan.small_church.is_some() && exterior == RoofMaterial::TimberInfill;
    if surface == RoofSurface::Interior {
        if boarded_belfry
            || plan.workplace.as_ref().is_some_and(|workplace| {
                matches!(
                    workplace.gable_material(),
                    crate::WorkplaceMaterial::Timber | crate::WorkplaceMaterial::UnpaintedTimber
                )
            })
        {
            BuildingLodMaterial::InteriorTimber
        } else {
            BuildingLodMaterial::InteriorPlaster
        }
    } else if boarded_belfry {
        BuildingLodMaterial::Roof(exterior)
    } else {
        plan.workplace.as_ref().map_or_else(
            || match exterior {
                RoofMaterial::TimberInfill => {
                    BuildingLodMaterial::Wall(WallMaterialClass::TimberInfill)
                }
                _ => roof_surface_material(exterior, surface),
            },
            |workplace| workplace.gable_material().render_material(),
        )
    }
}

pub(super) fn subtract_triangle(
    mut polygon: Vec<Vec3>,
    cut: [Vec3; 3],
    normal: Vec3,
) -> Vec<Vec<Vec3>> {
    let axes: [Vec3; 3] = std::array::from_fn(|edge| {
        let axis = normal.cross(cut[(edge + 1) % 3] - cut[edge]);
        if axis.dot(cut[(edge + 2) % 3] - cut[edge]) < 0.0 {
            -axis
        } else {
            axis
        }
    });
    let mut intersection = polygon.clone();
    for edge in 0..3 {
        intersection = split(&intersection, cut[edge], axes[edge]).0;
        if intersection.len() < 3 {
            return vec![polygon];
        }
    }
    let area = (1..intersection.len() - 1)
        .map(|i| {
            (intersection[i] - intersection[0])
                .cross(intersection[i + 1] - intersection[0])
                .length()
                * 0.5
        })
        .sum::<f32>();
    if area < MINIMUM_SEAM_AREA_SQUARE_METRES {
        return vec![polygon];
    }
    let mut result = Vec::new();
    for edge in 0..3 {
        let start = cut[edge];
        let axis = axes[edge];
        let (inside, outside) = split(&polygon, start, axis);
        if outside.len() >= 3 {
            result.push(outside);
        }
        polygon = inside;
        if polygon.len() < 3 {
            break;
        }
    }
    result
}

fn split(polygon: &[Vec3], origin: Vec3, axis: Vec3) -> (Vec<Vec3>, Vec<Vec3>) {
    let mut inside = Vec::new();
    let mut outside = Vec::new();
    for (index, &a) in polygon.iter().enumerate() {
        let b = polygon[(index + 1) % polygon.len()];
        let da = axis.dot(a - origin);
        let db = axis.dot(b - origin);
        if da >= 0.0 {
            inside.push(a);
        } else {
            outside.push(a);
        }
        if (da < 0.0) != (db < 0.0) {
            let intersection = a.lerp(b, da / (da - db));
            inside.push(intersection);
            outside.push(intersection);
        }
    }
    (inside, outside)
}
