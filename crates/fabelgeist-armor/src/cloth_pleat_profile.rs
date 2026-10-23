//! Uniform tailor-rest fold profiles shared by cloth relief and geometry.
use crate::Permille;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ClothPleatProfile {
    Fluted,
    RoundedGathers,
    Knife,
    Accordion,
}

impl ClothPleatProfile {
    pub const MOUTH_SHARE: Permille = Permille(800);
    pub const VALLEY_DEPTH_SHARE: Permille = Permille(200);
    /// Broad face followed by a short return, repeated in one direction.
    pub const KNIFE_FACE_SHARE: Permille = Permille(800);

    /// Native normalized repeat phase; result is a dimensionless depth share.
    pub(crate) fn height(self, phase: f32) -> f32 {
        match self {
            Self::Fluted => (1. - (std::f32::consts::TAU * phase).cos()) * 0.5,
            Self::Accordion => 1. - (2. * phase - 1.).abs(),
            Self::Knife => {
                let face = Self::KNIFE_FACE_SHARE.unit();
                if phase < face {
                    phase / face
                } else {
                    (1. - phase) / (1. - face)
                }
            }
            Self::RoundedGathers => {
                let mouth = Self::MOUTH_SHARE.unit();
                let valley = Self::VALLEY_DEPTH_SHARE.unit();
                let (local, low, high) = if phase < mouth {
                    (phase / mouth, valley, 1. - valley)
                } else {
                    ((phase - mouth) / (1. - mouth), valley, -valley)
                };
                low + high * (1. - (2. * local - 1.).powi(2)).max(0.).sqrt()
            }
        }
    }
}
