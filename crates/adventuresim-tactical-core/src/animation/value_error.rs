//! Structured rejection of invalid animation quantities and arithmetic.

use derive_more::Display;

/// Result of checked animation quantity admission or arithmetic.
pub type AnimationValueResult<T> = Result<T, AnimationValueError>;

/// The animation role whose numeric contract failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum AnimationQuantity {
    #[display("nonnegative length")]
    Length,
    #[display("nonnegative speed")]
    Speed,
    #[display("signed linear speed")]
    SignedSpeed,
    #[display("unit fraction")]
    UnitInterval,
    #[display("world position")]
    WorldPosition,
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
    #[display("baked stored-sample count")]
    BakedFrameCount,
    #[display("cycle phase")]
    CyclePhase,
    #[display("cycle displacement")]
    CycleDelta,
    #[display("blend contribution")]
    BlendWeight,
    #[display("dive influence")]
    DiveInfluence,
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
    #[error("baked animation channel has {actual} samples; expected {expected}")]
    BakedChannelSampleCount { expected: usize, actual: usize },
    #[error("{quantity} must be finite")]
    NonFinite { quantity: AnimationQuantity },
    #[error("{quantity} must be nonnegative")]
    Negative { quantity: AnimationQuantity },
    #[error("{quantity} must be positive")]
    NonPositive { quantity: AnimationQuantity },
    #[error("{quantity} must lie in [0, 1]")]
    OutsideUnitInterval { quantity: AnimationQuantity },
    #[error("{quantity} arithmetic overflowed")]
    Overflow { quantity: AnimationQuantity },
}

impl AnimationValueError {
    /// Retain a shared numeric rejection while identifying its semantic caller.
    pub fn for_quantity(self, quantity: AnimationQuantity) -> Self {
        match self {
            Self::BakedChannelSampleCount { .. } => self,
            Self::NonFinite { .. } => Self::NonFinite { quantity },
            Self::Negative { .. } => Self::Negative { quantity },
            Self::NonPositive { .. } => Self::NonPositive { quantity },
            Self::OutsideUnitInterval { .. } => Self::OutsideUnitInterval { quantity },
            Self::Overflow { .. } => Self::Overflow { quantity },
        }
    }
}

impl From<adventuresim_core::linear_speed::SpeedError> for AnimationValueError {
    fn from(error: adventuresim_core::linear_speed::SpeedError) -> Self {
        use adventuresim_core::linear_speed::SpeedError;
        let quantity = AnimationQuantity::Speed;
        match error {
            SpeedError::NonFinite => Self::NonFinite { quantity },
            SpeedError::Negative => Self::Negative { quantity },
            SpeedError::NonPositive => Self::NonPositive { quantity },
            SpeedError::Overflow => Self::Overflow { quantity },
        }
    }
}
