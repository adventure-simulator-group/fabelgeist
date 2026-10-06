//! Explicit architectural casement conversion preserves the native pose product.
use super::{CollisionCentreDatum, Scene};
use adventuresim_building_generator::spatial_geometry::{
    Architectural, GeometryError, PlanDirection, Radians, RigidRotation,
};
use adventuresim_building_generator::{WindowError, WindowSpec};
use bevy::math::{Quat, Vec2, Vec3};

/// Paired admitted scene leaf and native rotation, read through accessors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneWindowPose {
    leaf: WindowSpec<Scene>,
    native_rotation: RigidRotation,
}
impl SceneWindowPose {
    pub fn leaf(self) -> WindowSpec<Scene> {
        self.leaf
    }
    pub fn native_rotation(self) -> Quat {
        self.native_rotation.quaternion()
    }
}
impl CollisionCentreDatum {
    pub fn window(self, leaf: WindowSpec<Architectural>) -> Result<SceneWindowPose, WindowError> {
        let transform = self.native_transform();
        let direction = |value: PlanDirection<Architectural>| {
            let v = transform.rotation * Vec3::new(value.vector().x, 0.0, value.vector().y);
            PlanDirection::from_normalized(Vec2::new(v.x, v.z))
        };
        let admit = || -> Result<SceneWindowPose, GeometryError> {
            Ok(SceneWindowPose {
                native_rotation: RigidRotation::from_quaternion(
                    transform.rotation * Quat::from_rotation_y(leaf.closed_yaw_radians.radians()),
                )?,
                leaf: WindowSpec {
                    leaf: leaf.leaf,
                    opening: leaf.opening,
                    source: leaf.source,
                    closed_centre: self.point(leaf.closed_centre)?,
                    hinge_centre: self.point(leaf.hinge_centre)?,
                    size_metres: leaf.size_metres,
                    closed_yaw_radians: Radians::new(
                        self.orientation.yaw_radians() + leaf.closed_yaw_radians.radians(),
                    )?,
                    tangent: direction(leaf.tangent)?,
                    outward: direction(leaf.outward)?,
                    open_angle_radians: leaf.open_angle_radians,
                    barred: leaf.barred,
                },
            })
        };
        admit().map_err(|cause| WindowError {
            opening: leaf.opening,
            source_id: Some(leaf.source),
            cause: cause.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_input::BuildingOrientation;
    use adventuresim_building_generator::spatial_geometry::Position;
    use adventuresim_building_generator::{
        BuildingArchetype, BuildingProgram, compile_operable_windows, generate,
    };
    #[test]
    fn rotated_window_pose_retains_hinges_dimensions_and_native_product() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
        let origin = Position::<Architectural>::from_metres(Vec3::new(7.0, -2.0, 9.0)).unwrap();
        let centre = Position::<Scene>::from_metres(Vec3::new(-17.0, 3.5, 21.0)).unwrap();
        for angle in [0.0, 0.37, std::f32::consts::FRAC_PI_2] {
            let datum = CollisionCentreDatum::new(
                origin,
                centre,
                BuildingOrientation::from_radians(angle).unwrap(),
            )
            .unwrap();
            let transform = datum.native_transform();
            for original in compile_operable_windows(&plan).unwrap() {
                let pose = datum.window(original).unwrap();
                let scene = pose.leaf();
                assert_eq!(scene.source, original.source);
                assert_eq!(scene.opening, original.opening);
                assert_eq!(scene.size_metres, original.size_metres);
                assert_eq!(scene.barred, original.barred);
                assert_eq!(
                    scene.closed_centre.metres(),
                    transform.transform_point(original.closed_centre.metres() - origin.metres())
                );
                assert_eq!(
                    scene.hinge_centre.metres(),
                    transform.transform_point(original.hinge_centre.metres() - origin.metres())
                );
                assert_eq!(
                    pose.native_rotation(),
                    transform.rotation
                        * Quat::from_rotation_y(original.closed_yaw_radians.radians())
                );
                assert_eq!(
                    scene.tangent.spatial().vector(),
                    transform.rotation * original.tangent.spatial().vector()
                );
                assert_eq!(
                    scene.outward.spatial().vector(),
                    transform.rotation * original.outward.spatial().vector()
                );
                let native_open_hinge = scene.hinge_centre.metres()
                    + Quat::from_rotation_y(scene.open_angle_radians.radians())
                        * (scene.closed_centre.metres() - scene.hinge_centre.metres());
                let original_open = original.hinge_centre.metres()
                    + Quat::from_rotation_y(original.open_angle_radians.radians())
                        * (original.closed_centre.metres() - original.hinge_centre.metres());
                assert!(
                    native_open_hinge
                        .distance(transform.transform_point(original_open - origin.metres()))
                        < 0.00001
                );
                assert_eq!(
                    postcard::from_bytes::<WindowSpec<Scene>>(
                        &postcard::to_allocvec(&scene).unwrap()
                    )
                    .unwrap(),
                    scene
                );
            }
        }
    }
}
