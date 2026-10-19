//! Checked capture-file boundary for an explicit camera, lens and distance fog.

use std::path::Path;

use anyhow::{Result, ensure};
use bevy::camera::CameraProjection;
use bevy::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::stage::{self, Orbit};

/// Bound external camera documents before deserializing them.
const MAX_VIEW_BYTES: u64 = 4096;

/// A validated view carried as the renderer's native geometric types.
#[derive(Clone)]
pub struct CaptureView {
    pub orbit: Orbit,
    projection: PerspectiveProjection,
    fog: DistanceFog,
    transform: Transform,
}

/// JSON numerical boundary. Positions and fog distances use area-world metres;
/// the vertical field of view uses degrees. No framing heuristics apply here.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewDocument {
    eye_metres: [f32; 3],
    target_metres: [f32; 3],
    vertical_fov_degrees: f32,
    far_metres: f32,
    fog_start_metres: f32,
    fog_end_metres: f32,
}

impl CaptureView {
    pub fn read(path: &Path) -> Result<Self> {
        ensure!(
            std::fs::metadata(path)?.len() <= MAX_VIEW_BYTES,
            "capture view is too large"
        );
        let document: ViewDocument = serde_json::from_slice(&std::fs::read(path)?)?;
        document.admit()
    }

    pub fn framed(orbit: Orbit) -> Self {
        Self {
            orbit,
            projection: PerspectiveProjection::default(),
            fog: stage::distance_fog(),
            transform: orbit.transform(),
        }
    }

    pub fn lens(&self) -> impl Bundle {
        (
            Projection::Perspective(self.projection.clone()),
            self.fog.clone(),
        )
    }

    pub fn transform(&self) -> Transform {
        self.transform
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

impl ViewDocument {
    fn admit(self) -> Result<CaptureView> {
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
        Ok(CaptureView {
            orbit: Orbit {
                focus: target,
                radius,
                yaw: offset.x.atan2(offset.z),
                pitch: (offset.y / radius).clamp(-1.0, 1.0).asin(),
            },
            projection,
            transform,
            fog: DistanceFog {
                falloff: FogFalloff::Linear {
                    start: self.fog_start_metres,
                    end: self.fog_end_metres,
                },
                ..stage::distance_fog()
            },
        })
    }
}
