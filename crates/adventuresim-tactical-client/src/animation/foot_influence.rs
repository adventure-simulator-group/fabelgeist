//! Bounded left/right IK influence from semantic samples through pose solving.

use super::catalog;
use adventuresim_tactical_core::animation::{
    AnimationEvaluation, AnimationQuantity, AnimationValueError, AnimationValueResult,
    BlendWeight, ClipMirror, CycleDelta, CyclePhase, LocomotionProfile, PoseSampling,
    SemanticPose, UnitInterval, gait_support_weights, run_locomotion_profile,
    walk_locomotion_profile,
};
use bevy::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FootIkInfluence {
    left: UnitInterval,
    right: UnitInterval,
}

impl FootIkInfluence {
    pub(super) const NONE: Self = Self::new(UnitInterval::ZERO, UnitInterval::ZERO);
    pub(super) const BOTH: Self = Self::new(UnitInterval::ONE, UnitInterval::ONE);
    pub(super) const LEFT: Self = Self::new(UnitInterval::ONE, UnitInterval::ZERO);
    pub(super) const RIGHT: Self = Self::new(UnitInterval::ZERO, UnitInterval::ONE);

    pub(super) const fn new(left: UnitInterval, right: UnitInterval) -> Self {
        Self { left, right }
    }

    pub(super) fn left(self) -> UnitInterval {
        self.left
    }

    pub(super) fn right(self) -> UnitInterval {
        self.right
    }

    /// Preserve the authored weighted reduction before admitting its result.
    pub(super) fn from_evaluation(
        evaluation: &AnimationEvaluation,
    ) -> AnimationValueResult<Self> {
        let samples = if !evaluation.lower_body.is_empty() {
            &evaluation.lower_body
        } else if evaluation.action.is_empty() {
            &evaluation.base
        } else {
            return Ok(Self::NONE);
        };
        let mut result = Vec2::ZERO;
        let mut total = BlendWeight::ZERO;
        for sample in samples {
            let mut weights = match (sample.pose, sample.sampling) {
                (SemanticPose::IdleRelaxed, _) | (SemanticPose::CombatStance, _) => Self::BOTH,
                (
                    SemanticPose::StrafeCycle | SemanticPose::SkipCycle,
                    PoseSampling::Cycle { phase },
                ) => Self::combat_cycle(phase),
                (
                    SemanticPose::QuickstepForwardTakeoff
                    | SemanticPose::QuickstepRightTakeoff
                    | SemanticPose::QuickstepLeftTakeoff
                    | SemanticPose::QuickstepBackTakeoff,
                    PoseSampling::Timeline { .. },
                ) => Self::NONE,
                (SemanticPose::WalkContact, PoseSampling::Cycle { phase }) => {
                    let (left, right) = gait_support_weights(walk_locomotion_profile(), phase)?;
                    Self::new(left, right)
                }
                (SemanticPose::RunContact, PoseSampling::Cycle { phase }) => {
                    // IK becomes strong near authored contact, after descent;
                    // simulation support owns the complete stance interval.
                    let contact_profile = LocomotionProfile {
                        support_phase_radius: CycleDelta::from_cycles(0.11)?,
                        ..run_locomotion_profile()
                    };
                    let (left, right) = gait_support_weights(contact_profile, phase)?;
                    Self::new(left, right)
                }
                _ => Self::NONE,
            };
            if sample.mirror_lower_body == ClipMirror::Mirrored {
                weights = weights.mirrored();
            }
            // Vec2 owns the original component-wise multiply/add/divide kernel;
            // no native pair is retained in playback or passed to another API.
            let weight = sample.weight;
            let next = result
                + Vec2::new(weights.left.fraction(), weights.right.fraction()) * weight.factor();
            if !next.is_finite() {
                return Err(AnimationValueError::Overflow {
                    quantity: AnimationQuantity::BlendWeight,
                });
            }
            total = total.checked_add(weight)?;
            result = next;
        }
        let normalized = if total.is_absent() {
            Vec2::ZERO
        } else {
            result / total.factor()
        };
        if !normalized.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::BlendWeight,
            });
        }
        let normalized = normalized.clamp(Vec2::ZERO, Vec2::ONE);
        Ok(Self::new(
            UnitInterval::from_fraction(normalized.x)?,
            UnitInterval::from_fraction(normalized.y)?,
        ))
    }

    /// Authored contacts select one foot; pose interpolation supplies continuity.
    pub(super) fn combat_cycle(phase: CyclePhase) -> Self {
        let phase = phase.wrapped().cycles();
        if (catalog::COMBAT_FIRST_CONTACT_PHASE..catalog::COMBAT_SECOND_CONTACT_PHASE)
            .contains(&phase)
        {
            Self::LEFT
        } else {
            Self::RIGHT
        }
    }

    fn mirrored(self) -> Self {
        Self::new(self.right, self.left)
    }
}
