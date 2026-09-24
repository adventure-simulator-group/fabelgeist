//! Round journals and an inscribed collision cover that clears their bearings.
use crate::{BuildingLodMaterial, CollisionCuboid, LodMesh, ResolvedSolid};
use bevy::math::{EulerRot, Quat, Vec2, Vec3};

const SEGMENTS: usize = 32;
const COLLISION_DIAMETERS: usize = 8;

fn rotation(solid: &ResolvedSolid) -> Quat {
    Quat::from_euler(
        EulerRot::YXZ,
        solid.yaw_radians,
        solid.crossfall_radians,
        solid.longfall_radians,
    )
}

pub(crate) fn mesh(solid: &ResolvedSolid) -> LodMesh {
    let mut mesh = LodMesh::new(BuildingLodMaterial::Iron);
    let radius = solid.size.y.min(solid.size.z) * 0.5;
    let rotation = rotation(solid);
    let point = |p| solid.centre + rotation * p;
    for side in 0..SEGMENTS {
        let radial = [side, side + 1].map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / SEGMENTS as f32;
            Vec3::new(0.0, angle.cos(), angle.sin()) * radius
        });
        let ends = [Vec3::NEG_X, Vec3::X].map(|axis| axis * solid.size.x * 0.5);
        mesh.push_quad(
            [
                point(ends[0] + radial[0]),
                point(ends[1] + radial[0]),
                point(ends[1] + radial[1]),
                point(ends[0] + radial[1]),
            ],
            rotation * (radial[0] + radial[1]).normalize(),
            [Vec2::ZERO; 4],
        );
        for end in ends {
            mesh.push_triangle(
                [point(end), point(end + radial[0]), point(end + radial[1])],
                rotation * end.normalize(),
                [Vec2::ZERO; 3],
            );
        }
    }
    mesh
}

/// The radial cover is inscribed, so rotation cannot push a square corner
/// through a fixed bearing. Its maximum radial deficit is below nine percent.
pub(crate) fn collision(solid: &ResolvedSolid) -> Vec<CollisionCuboid> {
    let diameter = solid.size.y.min(solid.size.z);
    (0..COLLISION_DIAMETERS)
        .map(|index| {
            let rotation = rotation(solid)
                * Quat::from_rotation_x(
                    std::f32::consts::FRAC_PI_2 * index as f32 / COLLISION_DIAMETERS as f32,
                );
            let (yaw_radians, crossfall_radians, longfall_radians) =
                rotation.to_euler(EulerRot::YXZ);
            CollisionCuboid {
                source: solid.id,
                centre: solid.centre,
                size: Vec3::new(
                    solid.size.x,
                    diameter * std::f32::consts::FRAC_1_SQRT_2,
                    diameter * std::f32::consts::FRAC_1_SQRT_2,
                ),
                yaw_radians,
                crossfall_radians,
                longfall_radians,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inscribed_journal_cover_retains_at_least_ninety_one_percent_of_every_radius() {
        let solid = ResolvedSolid {
            id: crate::ResolvedItemId(1),
            owner: crate::GeometryOwnerId(1),
            centre: Vec3::new(2.0, 3.0, 4.0),
            size: Vec3::new(0.25, 0.08, 0.08),
            yaw_radians: 0.4,
            crossfall_radians: 0.3,
            longfall_radians: 0.2,
            role: crate::SolidRole::ChurchBellAxle,
            shape: crate::ResolvedSolidShape::CylinderAlongX,
            supported_by: Vec::new(),
        };
        let parts = collision(&solid);
        for sample in 0..720 {
            let angle = sample as f32 * std::f32::consts::TAU / 720.0;
            let point = solid.centre
                + rotation(&solid)
                    * Vec3::new(0.0, angle.cos(), angle.sin())
                    * solid.size.y
                    * 0.5
                    * 0.91;
            assert!(parts.iter().any(|part| {
                let rotation = Quat::from_euler(
                    EulerRot::YXZ,
                    part.yaw_radians,
                    part.crossfall_radians,
                    part.longfall_radians,
                );
                (rotation.inverse() * (point - part.centre))
                    .abs()
                    .cmple(part.size * 0.5)
                    .all()
            }));
        }
    }
}
