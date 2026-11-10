//! Distance violation, compliance scaled by substep, and constraint multipliers.
use super::projection::EffectiveInverseMass;

/// Compliance divided by squared substep duration, in inverse kilograms.
#[derive(Clone, Copy, Debug)]
pub struct CompliancePerSubstep(pub(super) f32);
impl From<f32> for CompliancePerSubstep {
    fn from(inverse_kilograms: f32) -> Self {
        Self(inverse_kilograms)
    }
}
/// Signed distance from the rest target, in metres.
#[derive(Clone, Copy, Debug)]
pub struct ConstraintViolation(f32);
impl From<f32> for ConstraintViolation {
    fn from(metres: f32) -> Self {
        Self(metres)
    }
}
impl ConstraintViolation {
    pub fn multiplier_delta(
        self,
        compliance: CompliancePerSubstep,
        multiplier: ConstraintMultiplier,
        response: EffectiveInverseMass,
    ) -> ConstraintMultiplier {
        ConstraintMultiplier((-self.0 - compliance.0 * multiplier.0) / response.0)
    }
}
/// Accumulated distance-constraint multiplier, in kilogram metres.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConstraintMultiplier(pub(super) f32);
impl ConstraintMultiplier {
    pub const ZERO: Self = Self(0.0);
}
impl std::ops::AddAssign for ConstraintMultiplier {
    fn add_assign(&mut self, delta: Self) {
        self.0 += delta.0;
    }
}
