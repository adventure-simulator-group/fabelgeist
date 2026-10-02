//! Hollow cast-bell geometry shared by architectural rendering and collision.
use crate::{BuildingLodMaterial, CollisionCuboid, LodMesh, ResolvedSolid};
use bevy::math::{Quat, Vec2, Vec3};

const SEGMENTS: usize = 20;
// Authored section of a small late-medieval cast bell, normalized to its bounds.
const SECTION: [(f32, f32); 13] = [
    (0.0, 0.5),
    (0.08, 0.48),
    (0.26, 0.35),
    (0.65, 0.24),
    (0.87, 0.21),
    (1.0, 0.07),
    (1.0, 0.0),
    (0.86, 0.0),
    (0.81, 0.16),
    (0.61, 0.19),
    (0.22, 0.30),
    (0.04, 0.43),
    (0.0, 0.44),
];

pub(crate) fn meshes(solid: &ResolvedSolid) -> Vec<LodMesh> {
    let mut mesh = LodMesh::new(BuildingLodMaterial::Bronze);
    let diameter = solid.size.x.min(solid.size.z);
    let bottom = solid.centre - Vec3::Y * solid.size.y * 0.5;
    for side in 0..SEGMENTS {
        let directions = [side, side + 1].map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / SEGMENTS as f32;
            Vec3::new(angle.sin(), 0.0, angle.cos())
        });
        for index in 0..SECTION.len() {
            let section = [SECTION[index], SECTION[(index + 1) % SECTION.len()]];
            let point = |ring: usize, side: usize| {
                bottom
                    + Vec3::Y * section[ring].0 * solid.size.y
                    + directions[side] * section[ring].1 * diameter
            };
            for positions in [
                [point(0, 0), point(0, 1), point(1, 1)],
                [point(0, 0), point(1, 1), point(1, 0)],
            ] {
                let normal = (positions[1] - positions[0])
                    .cross(positions[2] - positions[0])
                    .normalize_or_zero();
                if normal == Vec3::ZERO {
                    continue;
                }
                mesh.push_triangle(positions, normal, positions.map(|p| Vec2::new(p.x, p.y)));
            }
        }
    }
    let mut clapper = crate::furniture::builder::Builder::default();
    let profile = [
        (0.03, 0.035),
        (0.08, 0.075),
        (0.16, 0.035),
        (0.18, 0.02),
        (0.86, 0.02),
    ]
    .map(|(height, radius)| (height * solid.size.y, radius * diameter));
    clapper.turned(BuildingLodMaterial::Iron, Vec3::ZERO, &profile);
    let mut result = vec![mesh];
    for mut mesh in clapper.into_meshes() {
        for vertex in &mut mesh.vertices {
            vertex.position += bottom;
        }
        result.push(mesh);
    }
    result
}

pub(crate) fn collision(solid: &ResolvedSolid) -> Vec<CollisionCuboid> {
    let diameter = solid.size.x.min(solid.size.z);
    let bottom = solid.centre - Vec3::Y * solid.size.y * 0.5;
    let mut result = Vec::new();
    // Each wall chord stays outside the open mouth; the crown closes above it.
    for section in SECTION[..6].windows(2) {
        let a = Vec2::new(section[0].1 * diameter, section[0].0 * solid.size.y);
        let b = Vec2::new(section[1].1 * diameter, section[1].0 * solid.size.y);
        let wall_thickness = diameter * 0.055;
        for side in 0..SEGMENTS {
            let yaw = std::f32::consts::TAU * (side as f32 + 0.5) / SEGMENTS as f32;
            let radius = (a.x + b.x) * 0.5 - wall_thickness * 0.5;
            let centre =
                bottom + Vec3::new(yaw.sin() * radius, (a.y + b.y) * 0.5, yaw.cos() * radius);
            let slope = (b.x - a.x).atan2(b.y - a.y);
            let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(slope);
            let (yaw_radians, crossfall_radians, longfall_radians) =
                rotation.to_euler(bevy::math::EulerRot::YXZ);
            result.push(CollisionCuboid {
                source: solid.id,
                centre,
                size: Vec3::new(
                    2.0 * radius * (std::f32::consts::PI / SEGMENTS as f32).tan(),
                    a.distance(b),
                    wall_thickness,
                ),
                yaw_radians,
                crossfall_radians,
                longfall_radians,
            });
        }
    }
    result.push(CollisionCuboid {
        source: solid.id,
        centre: bottom + Vec3::Y * solid.size.y * 0.91,
        size: Vec3::new(diameter * 0.3, solid.size.y * 0.18, diameter * 0.3),
        yaw_radians: 0.0,
        crossfall_radians: 0.0,
        longfall_radians: 0.0,
    });
    for (height, rise, width) in [(0.1, 0.14, 0.15), (0.5, 0.72, 0.04)] {
        result.push(CollisionCuboid {
            source: solid.id,
            centre: bottom + Vec3::Y * height * solid.size.y,
            size: Vec3::new(diameter * width, solid.size.y * rise, diameter * width),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        });
    }
    result
}

/// Bell solids own a hollow cast section at every generator call site.
pub(crate) fn shape_for_role(role: crate::SolidRole) -> crate::ResolvedSolidShape {
    if role == crate::SolidRole::ChurchBell {
        crate::ResolvedSolidShape::BellShell
    } else if role == crate::SolidRole::ChurchBellAxle {
        crate::ResolvedSolidShape::CylinderAlongX
    } else {
        crate::ResolvedSolidShape::Cuboid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bell_mouth_is_open_in_geometry_and_collision_with_a_separate_clapper() {
        let plan = crate::generate(&crate::BuildingProgram::fixture(
            crate::BuildingArchetype::ParishChurch,
            42,
        ))
        .unwrap();
        let bell = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| solid.role == crate::SolidRole::ChurchBell)
            .unwrap();
        let bottom = bell.centre - Vec3::Y * bell.size.y * 0.5;
        let diameter = bell.size.x.min(bell.size.z);
        let batches = meshes(bell);
        assert!(
            batches
                .iter()
                .any(|mesh| mesh.material == BuildingLodMaterial::Bronze)
        );
        assert!(
            batches
                .iter()
                .any(|mesh| mesh.material == BuildingLodMaterial::Iron)
        );
        assert!(
            batches
                .iter()
                .flat_map(|mesh| &mesh.vertices)
                .all(|vertex| vertex.position.is_finite() && vertex.normal.is_finite())
        );
        let colliders = collision(bell);
        for height in [0.04, 0.2, 0.5, 0.75] {
            let point = bottom + Vec3::new(diameter * 0.12, bell.size.y * height, 0.0);
            assert!(
                !colliders.iter().any(|part| {
                    let rotation = Quat::from_euler(
                        bevy::math::EulerRot::YXZ,
                        part.yaw_radians,
                        part.crossfall_radians,
                        part.longfall_radians,
                    );
                    (rotation.inverse() * (point - part.centre))
                        .abs()
                        .cmplt(part.size * 0.5)
                        .all()
                }),
                "mouth and cavity must remain open around the clapper"
            );
        }
    }
}
