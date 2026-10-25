//! Round journals and an inscribed collision cover that clears their bearings.
use crate::{BuildingLodMaterial, CollisionCuboid, LodMesh, ResolvedSolid};
use bevy::math::{EulerRot, Quat, Vec2, Vec3};

const SEGMENTS: usize = 32;
const COLLISION_DIAMETERS: usize = 8;

fn rotation(solid: &ResolvedSolid) -> Quat {
    Quat::from_euler(
        EulerRot::YXZ,
        solid.yaw_radians.radians(),
        solid.crossfall_radians.radians(),
        solid.longfall_radians.radians(),
    )
}

pub(crate) fn mesh(solid: &ResolvedSolid) -> LodMesh {
    let mut mesh = LodMesh::new(BuildingLodMaterial::Iron);
    let radius = solid.size.metres().y.min(solid.size.metres().z) * 0.5;
    let rotation = rotation(solid);
    let point = |p| solid.centre.metres() + rotation * p;
    for side in 0..SEGMENTS {
        let radial = [side, side + 1].map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / SEGMENTS as f32;
            Vec3::new(0.0, angle.cos(), angle.sin()) * radius
        });
        let ends = [Vec3::NEG_X, Vec3::X].map(|axis| axis * solid.size.metres().x * 0.5);
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
pub(crate) fn collision(
    solid: &ResolvedSolid,
) -> Result<Vec<CollisionCuboid<crate::spatial_geometry::Architectural>>, crate::CollisionError> {
    let diameter = solid.size.metres().y.min(solid.size.metres().z);
    (0..COLLISION_DIAMETERS)
        .map(|index| {
            let rotation = rotation(solid)
                * Quat::from_rotation_x(
                    std::f32::consts::FRAC_PI_2 * index as f32 / COLLISION_DIAMETERS as f32,
                );
            let (yaw_radians, crossfall_radians, longfall_radians) =
                rotation.to_euler(EulerRot::YXZ);
            CollisionCuboid::from_metres(
                solid.id,
                solid.centre.metres(),
                Vec3::new(
                    solid.size.metres().x,
                    diameter * std::f32::consts::FRAC_1_SQRT_2,
                    diameter * std::f32::consts::FRAC_1_SQRT_2,
                ),
                yaw_radians,
                crossfall_radians,
                longfall_radians,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inscribed_journal_cover_retains_at_least_ninety_one_percent_of_every_radius() {
        let solid = crate::ResolvedSolid::new(
            crate::CollisionCuboid::<crate::Architectural>::from_metres(
                crate::ResolvedItemId(1),
                Vec3::new(2.0, 3.0, 4.0),
                Vec3::new(0.25, 0.08, 0.08),
                0.4,
                0.3,
                0.2,
            )
            .unwrap(),
            crate::GeometryOwnerId(1),
            crate::SolidRole::ChurchBellAxle,
            crate::ResolvedSolidShape::CylinderAlongX,
            Vec::new(),
        );
        let parts = collision(&solid).unwrap();
        for sample in 0..720 {
            let angle = sample as f32 * std::f32::consts::TAU / 720.0;
            let point = solid.centre.metres()
                + rotation(&solid)
                    * Vec3::new(0.0, angle.cos(), angle.sin())
                    * solid.size.metres().y
                    * 0.5
                    * 0.91;
            assert!(parts.iter().any(|part| {
                let rotation = Quat::from_euler(
                    EulerRot::YXZ,
                    part.yaw_radians.radians(),
                    part.crossfall_radians.radians(),
                    part.longfall_radians.radians(),
                );
                (rotation.inverse() * (point - part.centre.metres()))
                    .abs()
                    .cmple(part.size.metres() * 0.5)
                    .all()
            }));
        }
    }
}
