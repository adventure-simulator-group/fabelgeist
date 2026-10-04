//! Integration settings retain acceleration, drag and speed-limit units.
use super::SubstepDuration;
use fabelgeist_gpu::prelude::PassParameters;
use fabelgeist_math::{Vec3, Vec4};

/// Uniform acceleration in metres per second squared.
#[derive(Clone, Copy, Debug)]
pub struct GravityAcceleration(Vec3);
impl From<Vec3> for GravityAcceleration {
    fn from(acceleration: Vec3) -> Self {
        Self(acceleration)
    }
}
impl GravityAcceleration {
    pub fn earth() -> Self {
        Self(Vec3::new(0.0, -9.81, 0.0))
    }
    pub fn velocity_change(self, interval: SubstepDuration) -> Vec3 {
        Vec3::new(
            self.0.x * interval.0,
            self.0.y * interval.0,
            self.0.z * interval.0,
        )
    }
    pub fn bind(self, parameters: &mut PassParameters) {
        parameters.insert(
            "gravity".into(),
            Vec4::new(self.0.x, self.0.y, self.0.z, 0.0).into(),
        );
    }
}

/// Exponential velocity drag per second; native admission remains unchanged.
#[derive(Clone, Copy, Debug)]
pub struct DampingRate(f32);
impl From<f32> for DampingRate {
    fn from(per_second: f32) -> Self {
        Self(per_second)
    }
}
impl DampingRate {
    pub const DEFAULT: Self = Self(0.1);

    pub fn apply(self, velocity: Vec3, interval: SubstepDuration) -> Vec3 {
        let factor = (-self.0 * interval.0).exp();
        Vec3::new(
            velocity.x * factor,
            velocity.y * factor,
            velocity.z * factor,
        )
    }
    pub fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("damping".into(), self.0.into());
    }
}

/// Recovery ceiling on particle speed, in metres per second.
#[derive(Clone, Copy, Debug)]
pub struct ParticleSpeedLimit(f32);
impl From<f32> for ParticleSpeedLimit {
    fn from(metres_per_second: f32) -> Self {
        Self(metres_per_second)
    }
}
impl ParticleSpeedLimit {
    pub const DEFAULT: Self = Self(20.0);

    pub fn limit(self, velocity: Vec3) -> Vec3 {
        let speed = velocity.length();
        if speed > self.0 && speed > 0.0 {
            let factor = self.0 / speed;
            Vec3::new(
                velocity.x * factor,
                velocity.y * factor,
                velocity.z * factor,
            )
        } else {
            velocity
        }
    }
    pub fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("max_speed".into(), self.0.into());
    }
}
