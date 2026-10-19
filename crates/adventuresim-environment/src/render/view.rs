//! Checked perspective-camera geometry shared by native and browser captures.

use anyhow::{Result, ensure};
use bevy::{camera::CameraProjection, prelude::*};
use serde::Deserialize;
use serde_json::{Value, json};

/// Admitted renderer geometry; positions and target use area-world metres.
#[derive(Clone)]
pub struct ForestView {
    target: Vec3,
    projection: PerspectiveProjection,
    fog: DistanceFog,
    transform: Transform,
}

/// JSON numerical boundary. Positions and fog distances use area-world metres;
/// the vertical field of view uses degrees. No framing heuristics apply here.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForestViewDocument {
    eye_metres: [f32; 3],
    target_metres: [f32; 3],
    vertical_fov_degrees: f32,
    far_metres: f32,
    fog_start_metres: f32,
    fog_end_metres: f32,
}

impl ForestView {
    /// Trusted authored framing crosses the Bevy renderer boundary here.
    pub fn from_renderer(
        target: Vec3,
        transform: Transform,
        projection: PerspectiveProjection,
        fog: DistanceFog,
    ) -> Self {
        Self {
            target,
            transform,
            projection,
            fog,
        }
    }
    pub fn target(&self) -> Vec3 {
        self.target
    }
    pub fn transform(&self) -> Transform {
        self.transform
    }
    pub fn lens(&self) -> impl Bundle {
        (
            Projection::Perspective(self.projection.clone()),
            self.fog.clone(),
        )
    }
    /// Record the actual admitted lens, including the default framing lens.
    pub fn evidence(&self) -> Value {
        let fog = match self.fog.falloff {
            FogFalloff::Linear { start, end } => {
                json!({"mode": "linear", "start_metres": start, "end_metres": end})
            }
            FogFalloff::Exponential { density } => {
                json!({"mode": "exponential", "density_per_metre": density})
            }
            FogFalloff::ExponentialSquared { density } => {
                json!({"mode": "exponential_squared", "density_per_metre": density})
            }
            FogFalloff::Atmospheric {
                extinction,
                inscattering,
            } => {
                json!({"mode": "atmospheric", "extinction": extinction.to_array(), "inscattering": inscattering.to_array()})
            }
        };
        json!({
            "eye_metres": self.transform.translation.to_array(),
            "world_from_view": self.transform.to_matrix().to_cols_array(),
            "vertical_fov_radians": self.projection.fov,
            "near_metres": self.projection.near,
            "far_metres": self.projection.far,
            "fog": fog,
        })
    }
}

impl ForestViewDocument {
    pub fn admit(self, fog_style: DistanceFog) -> Result<ForestView> {
        let eye = Vec3::from_array(self.eye_metres);
        let target = Vec3::from_array(self.target_metres);
        let offset = eye - target;
        let radius = offset.length();
        ensure!(
            eye.is_finite()
                && target.is_finite()
                && radius.is_finite()
                && radius > 0.0
                && offset.xz().length_squared() > 0.0,
            "capture view needs finite distinct points and a nonvertical direction"
        );
        let fov = self.vertical_fov_degrees.to_radians();
        ensure!(
            fov.is_finite() && fov > 0.0 && fov < std::f32::consts::PI,
            "capture vertical field of view must be between zero and 180 degrees"
        );
        let projection = PerspectiveProjection {
            fov,
            far: self.far_metres,
            ..default()
        };
        ensure!(
            self.far_metres.is_finite() && self.far_metres > projection.near,
            "capture far plane must exceed the near plane"
        );
        let clip = projection.get_clip_from_view();
        ensure!(
            clip.is_finite() && clip.inverse().is_finite(),
            "capture view produces a nonfinite or singular projection"
        );
        ensure!(
            self.fog_start_metres.is_finite()
                && self.fog_end_metres.is_finite()
                && self.fog_start_metres >= 0.0
                && self.fog_end_metres > self.fog_start_metres,
            "capture fog needs finite ordered distances"
        );
        let transform = Transform::from_translation(eye).looking_at(target, Vec3::Y);
        ensure!(
            transform.to_matrix().is_finite(),
            "capture view produces a nonfinite transform"
        );
        Ok(ForestView {
            target,
            projection,
            transform,
            fog: DistanceFog {
                falloff: FogFalloff::Linear {
                    start: self.fog_start_metres,
                    end: self.fog_end_metres,
                },
                ..fog_style
            },
        })
    }
}
