//! Camera-owned composition, screen and filter contracts.

use fabelgeist_geometry::{GeometryError, SignedLength};
use adventuresim_core::linear_speed::SpeedError;
use adventuresim_tactical_core::animation::AnimationValueError;
use bevy::math::Vec2;
use fabelgeist_math::units::Ratio;

pub type CameraResult<T> = Result<T, CameraError>;

#[derive(Clone, Copy, Debug)]
pub struct FramingOffset {
    pub right: SignedLength,
    pub up: SignedLength,
}

/// Controller-relative right/up/planar-forward offsets. Forward here is the aim
/// direction, distinct from positive Z in a camera's view coordinates.
#[derive(Clone, Copy, Debug)]
pub struct MuzzleOffset {
    pub right: SignedLength,
    pub up: SignedLength,
    pub forward: SignedLength,
}

/// Finite nonnegative NDC half-extents; off-screen windows may exceed one.
#[derive(Clone, Copy, Debug)]
pub struct ScreenWindow { horizontal: Ratio, vertical: Ratio }

/// Finite signed NDC error; off-screen subjects may exceed one.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScreenError { horizontal: Ratio, vertical: Ratio }

/// Signed fraction per second retained by the analytic profile blend filter.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SignedBlendVelocity(f32);

#[derive(Debug, thiserror::Error)]
pub enum CameraError {
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Speed(#[from] SpeedError),
    #[error(transparent)]
    Animation(#[from] AnimationValueError),
    #[error("screen half-extents must be finite and nonnegative")]
    InvalidScreenWindow,
    #[error("screen error must be finite")]
    InvalidScreenError,
}

impl ScreenWindow {
    pub const fn from_ndc_half_extents(value: Vec2) -> CameraResult<Self> {
        if !value.x.is_finite() || !value.y.is_finite() || value.x < 0.0 || value.y < 0.0 {
            return Err(CameraError::InvalidScreenWindow);
        }
        Ok(Self { horizontal: Ratio::from_ratio(value.x), vertical: Ratio::from_ratio(value.y) })
    }

    pub fn ndc_half_extents(self) -> Vec2 {
        Vec2::new(self.horizontal.ratio(), self.vertical.ratio())
    }
}

impl ScreenError {
    pub(super) fn from_ndc(value: Vec2) -> CameraResult<Self> {
        if !value.is_finite() {
            return Err(CameraError::InvalidScreenError);
        }
        Ok(Self { horizontal: Ratio::from_ratio(value.x), vertical: Ratio::from_ratio(value.y) })
    }

    pub(crate) fn ndc(self) -> Vec2 {
        Vec2::new(self.horizontal.ratio(), self.vertical.ratio())
    }
}

impl SignedBlendVelocity {
    pub(super) fn from_fraction_per_second(value: f32) -> CameraResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: adventuresim_tactical_core::animation::AnimationQuantity::CameraBlendVelocity,
            }.into());
        }
        Ok(Self(value))
    }

    pub(super) fn fraction_per_second(self) -> f32 {
        self.0
    }
}
