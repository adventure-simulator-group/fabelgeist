//! Signed linear velocity in the scene's east/up/north axes.

use adventuresim_core::linear_speed::{SignedSpeed, SpeedResult};
use bevy::math::Vec3;

/// Finite scene-frame metres per second, including rest and reverse motion.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SceneVelocity {
    pub east: SignedSpeed,
    pub up: SignedSpeed,
    pub north: SignedSpeed,
}

impl SceneVelocity {
    pub const ZERO: Self = Self {
        east: SignedSpeed::ZERO,
        up: SignedSpeed::ZERO,
        north: SignedSpeed::ZERO,
    };

    /// Admit the native scene-vector output of a numerical or framework port.
    pub fn from_metres_per_second(value: Vec3) -> SpeedResult<Self> {
        Ok(Self {
            east: SignedSpeed::from_metres_per_second(value.x)?,
            up: SignedSpeed::from_metres_per_second(value.y)?,
            north: SignedSpeed::from_metres_per_second(value.z)?,
        })
    }

    /// Native X/Y/Z representation for numerical kernels and framework ports.
    pub fn metres_per_second(self) -> Vec3 {
        Vec3::new(
            self.east.metres_per_second(),
            self.up.metres_per_second(),
            self.north.metres_per_second(),
        )
    }
}
