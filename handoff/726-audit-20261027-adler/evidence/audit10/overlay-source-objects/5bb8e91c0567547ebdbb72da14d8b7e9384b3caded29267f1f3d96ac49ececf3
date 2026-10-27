//! Signed contact gradients, barycentric weights, and mass-weighted corrections.
use super::constraint::CompliancePerSubstep;
use super::mass::ParticleInverseMass;
use fabelgeist_math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionActivity {
    Inactive,
    Active,
}

/// Signed scalar derivative of contact separation with respect to a particle.
#[derive(Clone, Copy, Debug)]
pub struct ConstraintGradient(f32);
impl From<f64> for ConstraintGradient {
    fn from(gradient: f64) -> Self {
        Self(gradient as f32)
    }
}
impl ConstraintGradient {
    pub fn contribution(self, vector: Vec3) -> Vec3 {
        Vec3::new(vector.x * self.0, vector.y * self.0, vector.z * self.0)
    }
    pub fn inverse_response(self, mass: ParticleInverseMass) -> EffectiveInverseMass {
        EffectiveInverseMass(mass.0 * self.0 * self.0)
    }
}

/// Affine weight of a triangle vertex in a clipped surface sample. Its native
/// producer controls the affine coordinates; construction does not clamp them.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{BarycentricWeight, ConstraintGradient};
/// fn gradient(_: ConstraintGradient) {}
/// gradient(BarycentricWeight::ONE);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct BarycentricWeight(f32);
impl BarycentricWeight {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub fn contribution(self, vector: Vec3) -> Vec3 {
        Vec3::new(vector.x * self.0, vector.y * self.0, vector.z * self.0)
    }
    pub fn interpolate(self, other: Self, fraction: ClippingFraction) -> Self {
        Self(self.0 + (other.0 - self.0) * fraction.0)
    }
    pub fn inverse_response(self, mass: ParticleInverseMass) -> EffectiveInverseMass {
        EffectiveInverseMass(self.0.powi(2) * mass.0)
    }
    pub fn weighted_normal_speed(self, velocity: Vec3, normal: Vec3) -> IncomingNormalSpeed {
        IncomingNormalSpeed(velocity.dot(normal) * self.0)
    }
    pub fn mass_share(
        self,
        mass: ParticleInverseMass,
        response: EffectiveInverseMass,
    ) -> MassResponseShare {
        MassResponseShare(self.0 * mass.0 / response.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClippingFraction(f32);
impl From<f32> for ClippingFraction {
    fn from(fraction: f32) -> Self {
        Self(fraction)
    }
}
impl ClippingFraction {
    pub fn interpolate_point(self, point: Vec3, other: Vec3) -> Vec3 {
        let delta = other - point;
        point + Vec3::new(delta.x * self.0, delta.y * self.0, delta.z * self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EffectiveInverseMass(pub(super) f32);
impl EffectiveInverseMass {
    const CONTACT_THRESHOLD: f32 = 1e-12;
    const DISTANCE_THRESHOLD: f32 = 1e-12;
    pub fn contact_activity(self) -> ProjectionActivity {
        if self.0 <= Self::CONTACT_THRESHOLD {
            ProjectionActivity::Inactive
        } else {
            ProjectionActivity::Active
        }
    }
    pub fn layer_activity(self) -> ProjectionActivity {
        if self.0 <= 0.0 {
            ProjectionActivity::Inactive
        } else {
            ProjectionActivity::Active
        }
    }
    pub fn distance_activity(self) -> ProjectionActivity {
        if self.0 < Self::DISTANCE_THRESHOLD {
            ProjectionActivity::Inactive
        } else {
            ProjectionActivity::Active
        }
    }
    fn native_sum_term(self) -> f32 {
        self.0
    }
}
impl std::iter::Sum for EffectiveInverseMass {
    fn sum<I: Iterator<Item = Self>>(responses: I) -> Self {
        Self(responses.map(Self::native_sum_term).sum())
    }
}
impl std::ops::Add<CompliancePerSubstep> for EffectiveInverseMass {
    type Output = Self;
    fn add(self, compliance: CompliancePerSubstep) -> Self {
        Self(self.0 + compliance.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectionDepth(f32);
impl From<f32> for ProjectionDepth {
    fn from(metres: f32) -> Self {
        Self(metres)
    }
}
impl ProjectionDepth {
    pub fn activity(self) -> ProjectionActivity {
        if self.0 <= 0.0 {
            ProjectionActivity::Inactive
        } else {
            ProjectionActivity::Active
        }
    }
    pub fn contact_correction(
        self,
        gradient: ConstraintGradient,
        mass: ParticleInverseMass,
        response: EffectiveInverseMass,
    ) -> PositionCorrection {
        PositionCorrection(self.0 * gradient.0 * mass.0 / response.0)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct RelativeNormalSpeed(f32);
impl From<f32> for RelativeNormalSpeed {
    fn from(metres_per_second: f32) -> Self {
        Self(metres_per_second)
    }
}
impl RelativeNormalSpeed {
    pub fn activity(self) -> ProjectionActivity {
        if self.0 < 0.0 {
            ProjectionActivity::Active
        } else {
            ProjectionActivity::Inactive
        }
    }
    pub fn contact_correction(
        self,
        gradient: ConstraintGradient,
        mass: ParticleInverseMass,
        response: EffectiveInverseMass,
    ) -> NormalSpeedCorrection {
        NormalSpeedCorrection(self.0 * gradient.0 * mass.0 / response.0)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct IncomingNormalSpeed(f32);
impl IncomingNormalSpeed {
    pub fn incoming(self) -> Self {
        Self(self.0.min(0.0))
    }
    fn native_sum_term(self) -> f32 {
        self.0
    }
}
impl std::iter::Sum for IncomingNormalSpeed {
    fn sum<I: Iterator<Item = Self>>(speeds: I) -> Self {
        Self(speeds.map(Self::native_sum_term).sum())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MassResponseShare(f32);
impl MassResponseShare {
    pub fn position_correction(self, depth: ProjectionDepth) -> PositionCorrection {
        PositionCorrection(depth.0 * self.0)
    }
    pub fn speed_correction(self, speed: IncomingNormalSpeed) -> NormalSpeedCorrection {
        NormalSpeedCorrection(speed.0 * self.0)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PositionCorrection(pub(super) f32);
impl PositionCorrection {
    pub fn along(self, normal: Vec3) -> Vec3 {
        Vec3::new(normal.x * self.0, normal.y * self.0, normal.z * self.0)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct NormalSpeedCorrection(f32);
impl NormalSpeedCorrection {
    pub fn along(self, normal: Vec3) -> Vec3 {
        Vec3::new(normal.x * self.0, normal.y * self.0, normal.z * self.0)
    }
}
