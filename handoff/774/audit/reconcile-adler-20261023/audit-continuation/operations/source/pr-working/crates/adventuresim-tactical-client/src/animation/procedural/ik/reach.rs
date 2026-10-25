//! Checked limb dimensions retain the reach policy up to native geometry.

use super::*;

/// The two measured segment lengths from one immutable native chain snapshot.
#[derive(Debug, Clone, Copy)]
pub(in crate::animation::procedural) struct LimbSegments {
    pub(in crate::animation::procedural) upper: NonnegativeLength,
    pub(in crate::animation::procedural) lower: NonnegativeLength,
}

/// Reach is an operation on admitted segment geometry, not an eagerly stored
/// scalar. The native law-of-cosines intermediate may overflow before a later
/// clamp produces a valid finite target; it is never retained as a measurement.
#[derive(Debug, Clone, Copy)]
pub(in crate::animation::procedural) enum ReachLimit {
    Fixed(NonnegativeLength),
    Flexed {
        upper: NonnegativeLength,
        lower: NonnegativeLength,
        minimum_flexion: SignedAngleDegrees,
    },
    Extended {
        upper: NonnegativeLength,
        lower: NonnegativeLength,
    },
    Landing {
        upper: NonnegativeLength,
        lower: NonnegativeLength,
        authored: NonnegativeLength,
        compression: HeightDistance,
    },
}

impl From<NonnegativeLength> for ReachLimit {
    fn from(length: NonnegativeLength) -> Self {
        Self::Fixed(length)
    }
}

impl LimbSegments {
    pub(in crate::animation::procedural) fn measure(
        root: Vec3,
        knee: Vec3,
        end: Vec3,
    ) -> AnimationValueResult<Self> {
        let upper = NonnegativeLength::from_arithmetic(root.distance(knee))?;
        let lower = NonnegativeLength::from_arithmetic(knee.distance(end))?;
        Ok(Self { upper, lower })
    }
}

impl ReachLimit {
    /// Terrain confidence weights the geometric correction before the shared
    /// quarter-metre downward cap. Only the completed offset is retained.
    pub(super) fn terrain_pelvis_shift(
        self,
        upper: Vec3,
        target: Vec3,
        weight: UnitInterval,
    ) -> AnimationValueResult<SignedLength> {
        SignedLength::from_metres(
            (self.vertical_correction(upper, target).ok_or(
                AnimationValueError::InvertedBounds {
                    quantity: AnimationQuantity::Length,
                },
            )? * weight.fraction())
            .clamp(-0.25, 0.0),
        )
        .map_err(|_| AnimationValueError::Overflow {
            quantity: AnimationQuantity::HeightOffset,
        })
    }

    /// Ordinary authored IK clamps its geometric correction before weighting.
    pub(super) fn ordinary_pelvis_shift(
        self,
        upper: Vec3,
        target: Vec3,
        maximum_drop: NonnegativeLength,
        weight: UnitInterval,
    ) -> AnimationValueResult<SignedLength> {
        SignedLength::from_metres(
            self.vertical_correction(upper, target)
                .ok_or(AnimationValueError::InvertedBounds {
                    quantity: AnimationQuantity::Length,
                })?
                .clamp(-maximum_drop.metres(), 0.0)
                * weight.fraction(),
        )
        .map_err(|_| AnimationValueError::Overflow {
            quantity: AnimationQuantity::HeightOffset,
        })
    }

    fn vertical_correction(self, upper: Vec3, target: Vec3) -> Option<f32> {
        let reach = self.native_metres()?;
        let horizontal_distance = (target - upper).xz().length();
        let maximum_vertical = (reach * reach - horizontal_distance * horizontal_distance)
            .max(0.0)
            .sqrt();
        Some(target.y + maximum_vertical - upper.y)
    }

    /// Native coefficient only for the complete distance/projection/analytic
    /// solve kernels. An unordered landing interval has no supported solution;
    /// callers validate their completed target or owned result.
    pub(in crate::animation::procedural) fn native_metres(self) -> Option<f32> {
        Some(match self {
            Self::Fixed(length) => length.metres(),
            Self::Flexed {
                upper,
                lower,
                minimum_flexion,
            } => {
                let (upper, lower) = (upper.metres(), lower.metres());
                (upper * upper
                    + lower * lower
                    + 2.0 * upper * lower * minimum_flexion.radians().cos())
                .sqrt()
            }
            Self::Extended { upper, lower } => upper.metres() + lower.metres() - 0.0001,
            Self::Landing {
                upper,
                lower,
                authored,
                compression,
            } => {
                let reserved = maximum_reach(upper, lower).native_metres()?;
                let full = upper.metres() + lower.metres() - 0.0001;
                if reserved > full || reserved.is_nan() || full.is_nan() {
                    return None;
                }
                let released = authored.metres().clamp(reserved, full);
                let weight = smoothstep(
                    ik_tuning()
                        .landing_knee_reserve_release_compression_metres
                        .metres(),
                    ik_tuning()
                        .landing_knee_reserve_full_compression_metres
                        .metres(),
                    compression.metres(),
                );
                released.lerp(reserved, weight)
            }
        })
    }
}
