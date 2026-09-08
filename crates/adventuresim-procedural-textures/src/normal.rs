//! OpenGL tangent-space normals from physical height slopes in image coordinates.

use bevy::math::Vec3;

/// Image rows increase downwards; the OpenGL tangent-space Y axis points up.
/// A height increase down the image therefore tilts the normal towards +Y.
pub(crate) fn from_image_gradient(right_slope: f32, down_slope: f32) -> Vec3 {
    Vec3::new(-right_slope, down_slope, 1.0).normalize()
}
