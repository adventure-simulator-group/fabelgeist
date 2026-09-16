//! Sampled rigid bell sweeps against nearby collision solids and actual roof faces.
use super::*;
use bell_hanging::moving;

const CONTACT_TOLERANCE_METRES: f32 = 0.001;
const SWING_HALF_STEPS: i32 = 30;
type Triangle = [Vec3; 3];

pub(super) fn is_clear(plan: &BuildingPlan, bell: &ResolvedSolid, limit: f32) -> bool {
    let Some(axle) = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|part| part.owner == bell.owner && part.role == SolidRole::ChurchBellAxle)
    else {
        return false;
    };
    let pivot = Vec3::new(bell.centre.x, axle.centre.y, bell.centre.z);
    let moving_triangles = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|part| part.owner == bell.owner && moving(part.role))
        .flat_map(|part| crate::compile_solid_detail(plan, part).meshes)
        .flat_map(|mesh| {
            mesh.indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|indices| indices.map(|i| mesh.vertices[i as usize].position))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
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
        .flat_map(|part| crate::collision::collision_parts(plan, part))
        .filter(|part| {
            let bounds = part.bounds();
            pivot.clamp(bounds.min, bounds.max).distance_squared(pivot) <= radius_squared
        })
        .map(|part| {
            let rotation = Quat::from_euler(
                bevy::math::EulerRot::YXZ,
                part.yaw_radians,
                part.crossfall_radians,
                part.longfall_radians,
            );
            (
                part.centre,
                rotation.inverse(),
                part.size * 0.5 - Vec3::splat(CONTACT_TOLERANCE_METRES),
            )
        })
        .collect::<Vec<_>>();
    let roofs =
        plan.roof_assemblies
            .iter()
            .flat_map(|roof| {
                roof.faces
                    .iter()
                    .flat_map(crate::tessellate_roof_face)
                    .chain(roof.enclosure_faces.iter().flat_map(|face| {
                        crate::tessellate_roof_enclosure(face, &plan.wall_assemblies)
                    }))
            })
            .map(|triangle| triangle.positions)
            .filter(|triangle| {
                let (min, max) = bounds(*triangle);
                pivot.clamp(min, max).distance_squared(pivot) <= radius_squared
            })
            .collect::<Vec<_>>();
    for step in -SWING_HALF_STEPS..=SWING_HALF_STEPS {
        let rotation = Quat::from_rotation_x(limit * step as f32 / SWING_HALF_STEPS as f32);
        for triangle in &moving_triangles {
            let triangle = triangle.map(|p| pivot + rotation * (p - pivot));
            if fixed.iter().any(|&(centre, inverse, half)| {
                triangle_box(triangle.map(|p| inverse * (p - centre)), half)
            }) || roofs
                .iter()
                .any(|roof| triangles_intersect(triangle, *roof))
            {
                return false;
            }
        }
    }
    true
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

#[cfg(test)]
mod tests {
    use super::*;

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
        obstruction.size = Vec3::new(2.0, 0.03, 0.03);
        obstruction.yaw_radians = 0.3;
        obstruction.crossfall_radians = 0.2;
        obstruction.centre.y -= bell.size.y * 0.25;
        plan.resolved_geometry.solids.push(obstruction);
        assert!(!is_clear(&plan, &bell, std::f32::consts::FRAC_PI_6));
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
