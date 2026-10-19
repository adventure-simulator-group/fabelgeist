//! Authored locomotion roles and retained geometric cadence calibration.

use super::*;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use strum::EnumIter;

/// Per-step travel and irreducible stance slip measured from the loaded
/// authored foot curves. Until a motion and rig are both available,
/// presentation retains the shared simulation profile instead of guessing
/// from clip duration.
#[derive(Debug, Clone, Copy)]
pub(super) struct AuthoredStrideMeasurement {
    pub(super) step_distance: f32,
    pub(super) maximum_stance_slip: f32,
}

/// Periodic authored sample phase indexed by a uniformly advancing physical
/// gait phase. Values are deliberately left unwrapped so interpolation remains
/// continuous across the cycle seam; callers wrap only the final result.
#[derive(Debug, Clone)]
pub(super) struct AuthoredPhaseCurve {
    pub(super) authored_phases: Vec<CyclePhase>,
}

#[derive(Resource, Debug, Clone, Default)]
pub(super) struct AuthoredLocomotionStrides {
    pub(super) walk: Option<AuthoredStrideMeasurement>,
    pub(super) run: Option<AuthoredStrideMeasurement>,
    pub(super) strafe: Option<AuthoredStrideMeasurement>,
    pub(super) skip: Option<AuthoredStrideMeasurement>,
    pub(super) phase_curves: BTreeMap<CalibrationMotion, AuthoredPhaseCurve>,
    pub(super) measured_clips: BTreeMap<CalibrationMotion, AssetId<AnimationClip>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub(super) enum CalibrationMotion {
    Walk,
    Run,
    Strafe,
    Skip,
}

impl CalibrationMotion {
    pub(super) fn motion(self) -> MotionId {
        match self {
            Self::Walk => MotionId::WALK,
            Self::Run => MotionId::RUN,
            Self::Strafe => MotionId::STRAFE,
            Self::Skip => MotionId::SKIP,
        }
    }
}

impl AuthoredPhaseCurve {
    pub(super) fn sample(&self, physical_phase: CyclePhase) -> AnimationValueResult<CyclePhase> {
        if self.authored_phases.len() < 2 {
            return Ok(physical_phase.wrapped());
        }
        let phase = physical_phase.wrapped().cycles();
        let coordinate = phase * (self.authored_phases.len() - 1) as f32;
        let lower = coordinate.floor() as usize;
        let upper = (lower + 1).min(self.authored_phases.len() - 1);
        // The measured periodic curve is interpolated in native cycle coordinates.
        CyclePhase::from_cycles(
            self.authored_phases[lower]
                .cycles()
                .lerp(
                    self.authored_phases[upper].cycles(),
                    coordinate - lower as f32,
                )
                .rem_euclid(1.0),
        )
    }
}

impl AuthoredStrideMeasurement {
    pub(super) fn lerp(self, other: Self, amount: f32) -> Self {
        Self {
            step_distance: self.step_distance.lerp(other.step_distance, amount),
            maximum_stance_slip: self
                .maximum_stance_slip
                .lerp(other.maximum_stance_slip, amount),
        }
    }
}

impl AuthoredLocomotionStrides {
    pub(super) fn clear_motion(&mut self, motion: CalibrationMotion) {
        self.phase_curves.remove(&motion);
        self.measured_clips.remove(&motion);
        match motion {
            CalibrationMotion::Walk => self.walk = None,
            CalibrationMotion::Run => self.run = None,
            CalibrationMotion::Strafe => self.strafe = None,
            CalibrationMotion::Skip => self.skip = None,
        }
    }

    pub(super) fn invalidate_assets(&mut self, changed: &HashSet<AssetId<AnimationClip>>) {
        let motions: Vec<_> = self
            .measured_clips
            .iter()
            .filter(|(_, id)| changed.contains(id))
            .map(|(&motion, _)| motion)
            .collect();
        for motion in motions {
            self.clear_motion(motion);
        }
    }

    pub(super) fn sample_phase(
        &self,
        pose: SemanticPose,
        physical_phase: CyclePhase,
    ) -> AnimationValueResult<CyclePhase> {
        let motion = match pose {
            SemanticPose::WalkContact => CalibrationMotion::Walk,
            SemanticPose::RunContact => CalibrationMotion::Run,
            _ => return Ok(physical_phase.wrapped()),
        };
        self.phase_curves.get(&motion).map_or_else(
            || Ok(physical_phase.wrapped()),
            |curve| curve.sample(physical_phase),
        )
    }

    pub(super) fn ordinary(&self, speed: NonnegativeSpeed) -> Option<AuthoredStrideMeasurement> {
        let walk = self.walk?;
        let blend = ((speed.metres_per_second()
            - walk_locomotion_profile()
                .reference_speed
                .metres_per_second())
            / (run_locomotion_profile().reference_speed.metres_per_second()
                - walk_locomotion_profile()
                    .reference_speed
                    .metres_per_second()))
        .clamp(0.0, 1.0);
        if blend <= 0.0 {
            Some(walk)
        } else {
            Some(walk.lerp(self.run?, blend))
        }
    }

    pub(super) fn combat(&self, direction: Vec2) -> Option<AuthoredStrideMeasurement> {
        let strafe_weight = direction.x.abs();
        let skip_weight = direction.y.abs();
        let total = strafe_weight + skip_weight;
        if total <= 0.0 {
            return None;
        }
        let mut stride = AuthoredStrideMeasurement {
            step_distance: 0.0,
            maximum_stance_slip: 0.0,
        };
        if strafe_weight > 0.0 {
            let strafe = self.strafe?;
            stride.step_distance += strafe.step_distance * strafe_weight;
            stride.maximum_stance_slip += strafe.maximum_stance_slip * strafe_weight;
        }
        if skip_weight > 0.0 {
            let skip = self.skip?;
            stride.step_distance += skip.step_distance * skip_weight;
            stride.maximum_stance_slip += skip.maximum_stance_slip * skip_weight;
        }
        stride.step_distance /= total;
        stride.maximum_stance_slip /= total;
        Some(stride)
    }
}
