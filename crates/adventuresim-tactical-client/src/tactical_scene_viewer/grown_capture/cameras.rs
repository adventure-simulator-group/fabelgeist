//! Camera from the union of original source envelopes at actual grounded roots.

use super::*;
use fabelgeist_math::units::Metres;

pub(super) use catalog::Cameras;
pub(super) use framing::crown_distance;
pub(super) use support::{Inspector as SupportInspector, SupportedCameras, resolve, with_support};

mod catalog;
mod framing;
mod support;

const CAMERA_ROTATION_EPSILON: f32 = 1.0e-5;
pub(super) const FOV_DEGREES: f32 = 60.0;
const CROWN_EYE_DIRECTION: Vec3 = Vec3::new(1.0, 0.9, 1.0);
pub(super) const FOREGROUND_BPS: u16 = 500;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(in super::super) struct PlateCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

#[derive(Resource, Serialize)]
pub(in super::super) struct PerspectiveDepth {
    near: Metres,
    far: Metres,
}

impl PerspectiveDepth {
    pub(super) fn from_projection(projection: &Projection) -> Result<Self> {
        let Projection::Perspective(projection) = projection else {
            anyhow::bail!("review requires perspective depth");
        };
        ensure!(
            projection.near.is_finite()
                && projection.far.is_finite()
                && projection.near >= 0.0
                && projection.far > projection.near,
            "invalid production projection depth"
        );
        Ok(Self {
            near: Metres::from_metres(projection.near),
            far: Metres::from_metres(projection.far),
        })
    }
}

impl PlateCamera {
    pub(super) fn matches(
        self,
        camera: &bevy::prelude::Camera,
        transform: &GlobalTransform,
        projection: &Projection,
        depth: &PerspectiveDepth,
    ) -> bool {
        let Projection::Perspective(projection) = projection else {
            return false;
        };
        let expected = Transform::from_translation(self.position)
            .looking_at(self.target, self.up)
            .rotation;
        let rotation = transform.rotation();
        camera.is_active
            && camera.physical_viewport_size()
                == Some(super::super::capture_resolution::physical_pixels(PROFILE))
            && transform.translation() == self.position
            && (rotation.abs_diff_eq(expected, CAMERA_ROTATION_EPSILON)
                || rotation.abs_diff_eq(-expected, CAMERA_ROTATION_EPSILON))
            && projection.near == depth.near.metres()
            && projection.far == depth.far.metres()
            && projection.fov == FOV_DEGREES.to_radians()
            && projection.aspect_ratio == super::super::capture_resolution::CAPTURE_ASPECT_RATIO
    }

    pub(super) fn actual(
        self,
        camera: &Camera,
        transform: &GlobalTransform,
        projection: &Projection,
    ) -> Result<serde_json::Value> {
        let Projection::Perspective(projection) = projection else {
            anyhow::bail!("review requires perspective projection");
        };
        Ok(
            serde_json::json!({"expected": self, "physical_viewport": camera.physical_viewport_size(), "position_metres": transform.translation(), "rotation": transform.rotation(),
            "vertical_fov_radians": projection.fov, "aspect": projection.aspect_ratio, "near_metres": projection.near, "far_metres": projection.far}),
        )
    }
}
