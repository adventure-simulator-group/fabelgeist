//! Shared physical-pixel projection at the native geometry/camera boundary.

use bevy::{camera::primitives::Aabb, prelude::*};
use fabelgeist_math::units::Metres;

/// A camera inside a source envelope sees its nearest surface at this distance.
const NEAREST: Metres = Metres::from_metres(0.01);

/// Cached perspective scale; it carries no camera-entity or frame provenance.
#[derive(Clone, Copy, Debug)]
pub struct ForestLodProjection {
    focal: f32,
}

/// A finite, nonnegative projected source radius in physical image pixels.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
#[cfg_attr(feature = "prepared", serde(transparent))]
pub struct ProjectedRadiusPixels(f32);

impl ForestLodProjection {
    /// Native projection and physical viewport boundary. No view is fabricated.
    pub fn from_perspective(
        perspective: &PerspectiveProjection,
        physical_viewport: UVec2,
    ) -> Option<Self> {
        if physical_viewport.x == 0
            || physical_viewport.y == 0
            || !perspective.fov.is_finite()
            || perspective.fov <= 0.0
            || perspective.fov >= core::f32::consts::PI
            || !perspective.aspect_ratio.is_finite()
            || perspective.aspect_ratio <= 0.0
        {
            return None;
        }
        let focal = physical_viewport.y as f32 / (2.0 * (perspective.fov * 0.5).tan());
        (focal.is_finite() && focal > 0.0).then_some(Self { focal })
    }

    /// Native world-metre eye and Aabb, with the largest individual source radius.
    /// A spatial batch must not substitute its union-box radius here.
    pub fn projected_radius(
        self,
        eye: Vec3,
        aabb: &Aabb,
        radius: Metres,
    ) -> Option<ProjectedRadiusPixels> {
        let low = Vec3::from(aabb.min());
        let high = Vec3::from(aabb.max());
        if !eye.is_finite() || !low.is_finite() || !high.is_finite() || !low.cmple(high).all() {
            return None;
        }
        let distance = eye.distance(eye.clamp(low, high)).max(NEAREST.metres());
        ProjectedRadiusPixels::from_pixels(radius.metres() * self.focal / distance)
    }
}

impl ProjectedRadiusPixels {
    /// Physical-pixel numerical boundary, including a degenerate zero radius.
    pub fn from_pixels(pixels: f32) -> Option<Self> {
        (pixels.is_finite() && pixels >= 0.0).then_some(Self(pixels))
    }

    pub fn pixels(self) -> f32 {
        self.0
    }
}

/// Native source-envelope corners in the same metre frame. Preserve the radius
/// arithmetic used by actual prepared batches, independently of a batch union.
pub fn source_bounds_radius(low: Vec3, high: Vec3) -> Metres {
    Metres::from_metres((high - low).length() * 0.5)
}
