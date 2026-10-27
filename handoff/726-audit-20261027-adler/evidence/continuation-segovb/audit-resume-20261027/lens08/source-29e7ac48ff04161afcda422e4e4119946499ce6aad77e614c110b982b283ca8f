//! A physical support observation shared by terrain and presentation queries.
use super::SupportElevation;
use bevy::math::{Dir3, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceHit {
    pub elevation: SupportElevation,
    pub normal: Dir3,
}

impl SurfaceHit {
    /// Validate at the numeric geometry boundary; a zero normal is not support.
    pub fn from_geometry(elevation_metres: f32, normal: Vec3) -> Option<Self> {
        Some(Self {
            elevation: SupportElevation::from_metres(elevation_metres)?,
            normal: Dir3::new(normal).ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn support_rejects_nonfinite_elevation_and_degenerate_normal() {
        assert!(SurfaceHit::from_geometry(f32::NAN, Vec3::Y).is_none());
        assert!(SurfaceHit::from_geometry(0.0, Vec3::ZERO).is_none());
        assert!(SurfaceHit::from_geometry(0.0, Vec3::splat(f32::INFINITY)).is_none());
        let hit = SurfaceHit::from_geometry(-2.0, Vec3::Y * 3.0).unwrap();
        assert_eq!(hit.elevation.metres(), -2.0);
        assert_eq!(*hit.normal, Vec3::Y);
    }
}
