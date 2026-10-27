//! Structured rejection of invalid animation quantities and arithmetic.

use derive_more::Display;

/// Result of checked animation quantity admission or arithmetic.
pub type AnimationValueResult<T> = Result<T, AnimationValueError>;

/// The animation role whose numeric contract failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum AnimationQuantity {
    #[display("selected-axis calibration metre position")]
    CalibrationPosition,
    #[display("authored contact-height scale")]
    ContactHeightScale,
    #[display("nonnegative length")]
    Length,
    #[display("nonnegative speed")]
    Speed,
    #[display("signed linear speed")]
    SignedSpeed,
    #[display("foot support observation")]
    SupportObservation,
    #[display("unit fraction")]
    UnitInterval,

    #[display("settle progress")]
    SettleProgress,
    #[display("knee-target amplification")]
    KneeTargetAmplification,
    #[display("world position")]
    WorldPosition,
    #[display("world velocity")]
    WorldVelocity,
    #[display("body orientation")]
    BodyOrientation,
    #[display("animation sampling frequency")]
    SampleRate,
    #[display("native fixed-clock interval")]
    NativeSampleInterval,
    #[display("contact timing offset seconds")]
    ContactTimingOffset,
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
    #[display("authored attack pose coordinate")]
    AttackPoseCoordinate,
    #[display("attack continuation tangent")]
    ContinuationTangent,
    #[display("attack continuation readiness knot")]
    ContinuationReadiness,
    #[display("action progress displacement")]
    ActionProgressDelta,
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
    #[display("muscle tone")]
    MuscleTone,
    #[display("angular stiffness")]
    AngularStiffness,
    #[display("forward lunge acceleration")]
    LungeAcceleration,
    #[display("reference acceleration")]
    ReferenceAcceleration,
    #[display("response rate")]
    ResponseRate,
    #[display("phase correction rate")]
    PhaseCorrectionRate,
    #[display("phase error limit")]
    PhaseErrorLimit,
    #[display("authored step cadence")]
    StepCadence,
    #[display("coalesced animation-event budget")]
    CoalescedEventBudget,
    #[display("inertial lean gain")]
    InertialLeanGain,
    #[display("impact compression compliance")]
    ImpactCompressionCompliance,
    #[display("look distribution")]
    LookDistribution,
    #[display("camera blend velocity")]
    CameraBlendVelocity,
    #[display("response shaping seconds")]
    ResponseShapingTime,
    #[display("downed half-turn coordinate")]
    DownedFacing,
    #[display("downed half-turn step")]
    DownedFacingStep,
    #[display("downed roll command")]
    DownedRollMotion,
    #[display("body angle")]
    BodyAngle,
    #[display("signed angle")]
    SignedAngle,
    #[display("body angular velocity")]
    BodyAngularVelocity,
    #[display("body-local velocity")]
    BodyLocalVelocity,
    #[display("body-local acceleration")]
    BodyLocalAcceleration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AnimationValueError {
    #[error("authored cadence smoothing transition cannot begin below zero steps per second")]
    CadenceTransitionBelowZero,
    #[error("{quantity} is unavailable")]
    UnavailableObservation { quantity: AnimationQuantity },
    #[error(
        "attack continuation readiness must lie strictly between zero and contact at phase 0.5"
    )]
    InvalidContinuationReadiness,
    #[error("authored phase curve must close by one forward or reverse cycle")]
    InvalidPhaseCurveSeam,
    #[error("authored phase calibration requires at least three intervals; got {actual}")]
    InsufficientPhaseIntervals { actual: u32 },
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
    #[error("{quantity} must lie in [-1, 1]")]
    OutsideSignedUnitInterval { quantity: AnimationQuantity },
    #[error("{quantity} bounds are inverted")]
    InvertedBounds { quantity: AnimationQuantity },
    #[error("{quantity} arithmetic overflowed")]
    Overflow { quantity: AnimationQuantity },
}

impl AnimationValueError {
    /// Retain a shared numeric rejection while identifying its semantic caller.
    pub fn for_quantity(self, quantity: AnimationQuantity) -> Self {
        match self {
            Self::CadenceTransitionBelowZero
            | Self::BakedChannelSampleCount { .. }
            | Self::InvalidContinuationReadiness
            | Self::InvalidPhaseCurveSeam
            | Self::InsufficientPhaseIntervals { .. } => self,
            Self::UnavailableObservation { .. } => Self::UnavailableObservation { quantity },
            Self::NonFinite { .. } => Self::NonFinite { quantity },
            Self::Negative { .. } => Self::Negative { quantity },
            Self::NonPositive { .. } => Self::NonPositive { quantity },
            Self::OutsideUnitInterval { .. } => Self::OutsideUnitInterval { quantity },
            Self::OutsideSignedUnitInterval { .. } => Self::OutsideSignedUnitInterval { quantity },
            Self::InvertedBounds { .. } => Self::InvertedBounds { quantity },
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
            SpeedError::Fraction(error) => Self::from(error),
        }
    }
}

impl From<adventuresim_core::elapsed::ElapsedError> for AnimationValueError {
    fn from(error: adventuresim_core::elapsed::ElapsedError) -> Self {
        use adventuresim_core::elapsed::ElapsedError;
        let quantity = AnimationQuantity::Elapsed;
        match error {
            ElapsedError::NonFiniteContactOffset => Self::NonFinite {
                quantity: AnimationQuantity::ContactTimingOffset,
            },
            ElapsedError::ContactOffsetOverflow => Self::Overflow {
                quantity: AnimationQuantity::ContactTimingOffset,
            },
            ElapsedError::NonFinite => Self::NonFinite { quantity },
            ElapsedError::Negative => Self::Negative { quantity },
            ElapsedError::Overflow | ElapsedError::NativeRange => Self::Overflow { quantity },
        }
    }
}

impl From<adventuresim_core::unit_interval::FractionError> for AnimationValueError {
    fn from(error: adventuresim_core::unit_interval::FractionError) -> Self {
        use adventuresim_core::unit_interval::FractionError;
        let quantity = AnimationQuantity::UnitInterval;
        match error {
            FractionError::NonFinite => Self::NonFinite { quantity },
            FractionError::OutsideUnitInterval => Self::OutsideUnitInterval { quantity },
        }
    }
}
