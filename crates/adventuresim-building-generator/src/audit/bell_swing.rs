//! Sampled rigid bell sweeps against nearby collision solids and actual roof faces.
use super::*;
use crate::spatial_geometry::Radians;
use bell_hanging::moving;
use std::result::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BellSwingAssessment {
    Clear,
    MissingAxle {
        bell: ResolvedItemId,
    },
    Blocked {
        bell: ResolvedItemId,
        source: ResolvedItemId,
    },
}

const CONTACT_TOLERANCE_METRES: f32 = 0.001;
const SWING_HALF_STEPS: i32 = 30;
type Triangle = [Vec3; 3];

// Mesh triangles stay native inside the sampled rigid sweep kernel. Fixed
// cuboids and roof triangles retain their source IDs for obstruction reports.
pub(super) fn is_clear(
    plan: &BuildingPlan,
    bell: &ResolvedSolid,
    limit: Radians,
) -> Result<BellSwingAssessment, crate::GenerationError> {
    let Some(axle) = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|part| part.owner == bell.owner && part.role == SolidRole::ChurchBellAxle)
    else {
        return Ok(BellSwingAssessment::MissingAxle { bell: bell.id });
    };
    let pivot = Vec3::new(
        bell.centre.metres().x,
        axle.centre.metres().y,
        bell.centre.metres().z,
    );
    let moving_triangles = moving_mesh_triangles(plan, bell)?;
    let radius_squared = moving_triangles
        .iter()
        .flatten()
        .map(|p| p.distance_squared(pivot))
        .fold(0.0_f32, f32::max);
    let fixed = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|part| !(part.owner == bell.owner && moving(part.role)))
        .map(|part| crate::collision::collision_parts(plan, part))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .map(|part| part.bounds().map(|bounds| (part, bounds)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(_, bounds)| {
            pivot
                .clamp(bounds.min().metres(), bounds.max().metres())
                .distance_squared(pivot)
                <= radius_squared
        })
        .map(|(part, _)| part)
        .map(|part| {
            let rotation = Quat::from_euler(
                bevy::math::EulerRot::YXZ,
                part.yaw_radians.radians(),
                part.crossfall_radians.radians(),
                part.longfall_radians.radians(),
            );
            (
                part.source,
                part.centre.metres(),
                rotation.inverse(),
                part.size.metres() * 0.5 - Vec3::splat(CONTACT_TOLERANCE_METRES),
            )
        })
        .collect::<Vec<_>>();
    let roofs = nearby_roof_triangles(plan, pivot, radius_squared);
    for step in -SWING_HALF_STEPS..=SWING_HALF_STEPS {
        let rotation =
            Quat::from_rotation_x(limit.radians() * step as f32 / SWING_HALF_STEPS as f32);
        for triangle in &moving_triangles {
            let triangle = triangle.map(|p| pivot + rotation * (p - pivot));
            let source = fixed
                .iter()
                .find(|&&(_, centre, inverse, half)| {
                    triangle_box(triangle.map(|p| inverse * (p - centre)), half)
                })
                .map(|entry| entry.0)
                .or_else(|| {
                    roofs
                        .iter()
                        .find(|(_, roof)| triangles_intersect(triangle, *roof))
                        .map(|entry| entry.0)
                });
            if let Some(source) = source {
                return Ok(BellSwingAssessment::Blocked {
                    bell: bell.id,
                    source,
                });
            }
        }
    }
    Ok(BellSwingAssessment::Clear)
}

fn moving_mesh_triangles(
    plan: &BuildingPlan,
    bell: &ResolvedSolid,
) -> Result<Vec<Triangle>, crate::GenerationError> {
    let mut triangles = Vec::new();
    for part in plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|part| part.owner == bell.owner && moving(part.role))
    {
        for mesh in crate::compile_solid_detail(plan, part)?.meshes {
            triangles.extend(
                mesh.indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|indices| indices.map(|i| mesh.vertices[i as usize].position)),
            );
        }
    }
    Ok(triangles)
}

fn bounds(triangle: Triangle) -> (Vec3, Vec3) {
    (
        triangle
            .into_iter()
            .fold(Vec3::splat(f32::INFINITY), Vec3::min),
        triangle
            .into_iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max),
    )
}

fn edges(triangle: Triangle) -> [Vec3; 3] {
    [
        triangle[1] - triangle[0],
        triangle[2] - triangle[1],
        triangle[0] - triangle[2],
    ]
}

fn projection(triangle: Triangle, axis: Vec3) -> (f32, f32) {
    triangle
        .map(|p| p.dot(axis))
        .into_iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
            (min.min(value), max.max(value))
        })
}

/// Triangle/box separating axes also detect a narrow beam crossing a triangle
/// whose three vertices all lie outside the beam.
fn triangle_box(triangle: Triangle, half: Vec3) -> bool {
    let (min, max) = bounds(triangle);
    if max.cmple(-half).any() || min.cmpge(half).any() {
        return false;
    }
    let edges = edges(triangle);
    std::iter::once(edges[0].cross(edges[1]))
        .chain(
            edges
                .into_iter()
                .flat_map(|edge| [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| edge.cross(axis))),
        )
        .all(|axis| {
            let (min, max) = projection(triangle, axis);
            let extent = half.dot(axis.abs());
            min <= extent && max >= -extent
        })
}

fn triangles_intersect(left: Triangle, right: Triangle) -> bool {
    let (a_min, a_max) = bounds(left);
    let (b_min, b_max) = bounds(right);
    if a_min.cmpgt(b_max).any() || b_min.cmpgt(a_max).any() {
        return false;
    }
    let a = edges(left);
    let b = edges(right);
    let normals = [a[0].cross(a[1]), b[0].cross(b[1])];
    normals
        .into_iter()
        .chain(
            a.into_iter()
                .flat_map(|edge| b.map(|other| edge.cross(other))),
        )
        .chain(a.into_iter().chain(b).map(|edge| normals[0].cross(edge)))
        .all(|axis| {
            let (a_min, a_max) = projection(left, axis);
            let (b_min, b_max) = projection(right, axis);
            a_min <= b_max && b_min <= a_max
        })
}

fn nearby_roof_triangles(
    plan: &BuildingPlan,
    pivot: Vec3,
    radius_squared: f32,
) -> Vec<(crate::ResolvedItemId, Triangle)> {
    let mut roofs = Vec::new();
    for roof in &plan.roof_assemblies {
        for face in &roof.faces {
            roofs.extend(
                crate::tessellate_roof_face(face)
                    .into_iter()
                    .map(|triangle| (face.id, triangle.positions)),
            );
        }
        for face in &roof.enclosure_faces {
            roofs.extend(
                crate::tessellate_roof_enclosure(face, &plan.wall_assemblies)
                    .into_iter()
                    .map(|triangle| (face.id, triangle.positions)),
            );
        }
    }
    roofs.retain(|(_, triangle)| {
        let (min, max) = bounds(*triangle);
        pivot.clamp(min, max).distance_squared(pivot) <= radius_squared
    });
    roofs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_axle_reports_the_bell_identity() {
        let mut plan = crate::generate(&crate::BuildingProgram::fixture(
            BuildingArchetype::ParishChurch,
            42,
        ))
        .unwrap();
        let bell = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| solid.role == SolidRole::ChurchBell)
            .unwrap()
            .clone();
        plan.resolved_geometry
            .solids
            .retain(|solid| solid.owner != bell.owner || solid.role != SolidRole::ChurchBellAxle);
        assert_eq!(
            is_clear(&plan, &bell, Radians::QUARTER_TURN).unwrap(),
            BellSwingAssessment::MissingAxle { bell: bell.id }
        );
    }

    #[test]
    fn narrow_rotated_foreign_beam_blocks_bell_sweep() {
        let mut plan = crate::generate(&crate::BuildingProgram::fixture(
            BuildingArchetype::ParishChurch,
            42,
        ))
        .unwrap();
        let bell = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.role == SolidRole::ChurchBell)
            .unwrap()
            .clone();
        let mut obstruction = bell.clone();
        obstruction.id = ResolvedItemId(u64::MAX);
        obstruction.owner = crate::GeometryOwnerId(u32::MAX);
        obstruction.role = SolidRole::BeamJoist;
        obstruction.shape = crate::ResolvedSolidShape::Cuboid;
        obstruction.size =
            crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(2.0, 0.03, 0.03))
                .unwrap();
        obstruction.yaw_radians = crate::spatial_geometry::Radians::new(0.3).unwrap();
        obstruction.crossfall_radians = crate::spatial_geometry::Radians::new(0.2).unwrap();
        {
            let mut native_geometry = obstruction.centre.metres();
            native_geometry.y -= bell.size.metres().y * 0.25;
            obstruction.centre =
                crate::spatial_geometry::Position::from_metres(native_geometry).unwrap();
        };
        plan.resolved_geometry.solids.push(obstruction);
        assert!(
            matches!(is_clear(&plan, &bell, Radians::new(std::f32::consts::FRAC_PI_6).unwrap()).unwrap(), BellSwingAssessment::Blocked { bell: id, source } if id == bell.id && source == ResolvedItemId(u64::MAX))
        );
    }

    #[test]
    fn beam_intersection_does_not_require_an_enclosed_mesh_vertex() {
        assert!(triangle_box(
            [
                Vec3::new(-2.0, 0.0, 0.0),
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 2.0)
            ],
            Vec3::splat(0.01)
        ));
        assert!(!triangle_box(
            [
                Vec3::new(-2.0, 1.0, 0.0),
                Vec3::new(2.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 2.0)
            ],
            Vec3::splat(0.01)
        ));
    }
}
