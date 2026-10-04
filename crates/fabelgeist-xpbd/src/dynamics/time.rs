//! Frame and substep seconds retain their native words and admission policies.
use super::SubstepCount;
use fabelgeist_gpu::prelude::PassParameters;
use fabelgeist_math::Vec3;

/// Seconds for a complete driver step, before subdivision.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{StepDuration, SubstepDuration};
/// fn frame(_: StepDuration) {}
/// frame(SubstepDuration::from(0.01));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct StepDuration(f32);
impl From<f32> for StepDuration {
    fn from(seconds: f32) -> Self {
        Self(seconds)
    }
}
impl StepDuration {
    pub fn activity(self) -> StepActivity {
        if self.0 <= 0.0 {
            StepActivity::Inactive
        } else {
            StepActivity::Active
        }
    }
    pub fn finite_activity(self) -> StepActivity {
        if !self.0.is_finite() || self.0 <= 0.0 {
            StepActivity::Inactive
        } else {
            StepActivity::Active
        }
    }
    pub fn for_substeps(self, count: SubstepCount) -> SubstepDuration {
        SubstepDuration(self.0 / count.0 as f32)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepActivity {
    Inactive,
    Active,
}

/// Seconds for one prediction/constraint/finalization interval.
#[derive(Clone, Copy, Debug)]
pub struct SubstepDuration(pub(super) f32);
impl From<f32> for SubstepDuration {
    fn from(seconds: f32) -> Self {
        Self(seconds)
    }
}
impl From<SubstepDuration> for u32 {
    fn from(interval: SubstepDuration) -> Self {
        interval.0.to_bits()
    }
}
impl SubstepDuration {
    pub fn bind(self, parameters: &mut PassParameters) {
        parameters.insert("substep".into(), self.0.into());
    }
    pub fn displacement(self, velocity: Vec3) -> Vec3 {
        Vec3::new(
            velocity.x * self.0,
            velocity.y * self.0,
            velocity.z * self.0,
        )
    }
    pub fn velocity(self, displacement: Vec3) -> Vec3 {
        Vec3::new(
            displacement.x / self.0,
            displacement.y / self.0,
            displacement.z / self.0,
        )
    }
}
