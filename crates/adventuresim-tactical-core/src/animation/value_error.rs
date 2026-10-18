//! Structured rejection of invalid animation quantities and arithmetic.

use derive_more::Display;

/// Result of checked animation quantity admission or arithmetic.
pub type AnimationValueResult<T> = Result<T, AnimationValueError>;

/// The animation role whose numeric contract failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum AnimationQuantity {
    #[display("world velocity")]
    WorldVelocity,
    #[display("body orientation")]
    BodyOrientation,
    #[display("elapsed animation seconds")]
    Elapsed,
    #[display("landing impact speed")]
    LandingImpactSpeed,
    #[display("clip duration")]
    ClipDuration,
    #[display("clip sample time")]
    ClipTime,
    #[display("cycle phase")]
    CyclePhase,
    #[display("blend contribution")]
    BlendWeight,
    #[display("pose interpolation fraction")]
    PoseInterpolation,
    #[display("height distance")]
    HeightDistance,
    #[display("height offset")]
    HeightOffset,
    #[display("height rate")]
    HeightSpeed,
    #[display("braking lean scale")]
    BrakingLeanScale,
    #[display("body angle")]
    BodyAngle,
    #[display("body angular velocity")]
    BodyAngularVelocity,
    #[display("body-local velocity")]
    BodyLocalVelocity,
    #[display("body-local acceleration")]
    BodyLocalAcceleration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AnimationValueError {
    #[error("{quantity} must be finite")]
    NonFinite { quantity: AnimationQuantity },
    #[error("{quantity} must be nonnegative")]
    Negative { quantity: AnimationQuantity },
    #[error("{quantity} must lie in [0, 1]")]
    OutsideUnitInterval { quantity: AnimationQuantity },
    #[error("{quantity} arithmetic overflowed")]
    Overflow { quantity: AnimationQuantity },
}
