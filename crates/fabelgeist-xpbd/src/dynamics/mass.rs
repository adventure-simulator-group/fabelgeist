//! Kilogram mass, inverse kilogram response, and kilogram-per-square-metre input.
use super::constraint::ConstraintMultiplier;
use super::projection::{EffectiveInverseMass, PositionCorrection};
use fabelgeist_math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MassValidity {
    FiniteNonnegative,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArealDensityValidity {
    PositiveFinite,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleMobility {
    Prescribed,
    Dynamic,
}

/// Inverse kilograms. Native construction preserves all float words; CPU
/// admission is explicit and does not silently rewrite native GPU state.
///
/// ```compile_fail
/// use fabelgeist_xpbd::{ParticleInverseMass, ParticleMass};
/// fn inverse(_: ParticleInverseMass) {}
/// inverse(ParticleMass::from(1.0));
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleInverseMass(pub(super) f32);
impl From<f32> for ParticleInverseMass {
    fn from(inverse_kilograms: f32) -> Self {
        Self(inverse_kilograms)
    }
}
impl From<ParticleInverseMass> for u32 {
    fn from(mass: ParticleInverseMass) -> Self {
        mass.0.to_bits()
    }
}
impl ParticleInverseMass {
    pub const PINNED: Self = Self(0.0);
    pub const UNIT_MASS: Self = Self(1.0);
    pub fn validity(self) -> MassValidity {
        if self.0.is_finite() && self.0 >= 0.0 {
            MassValidity::FiniteNonnegative
        } else {
            MassValidity::Invalid
        }
    }
    pub fn mobility(self) -> ParticleMobility {
        if self.0 == 0.0 {
            ParticleMobility::Prescribed
        } else {
            ParticleMobility::Dynamic
        }
    }
    pub fn distance_correction(self, multiplier: ConstraintMultiplier) -> PositionCorrection {
        PositionCorrection(self.0 * multiplier.0)
    }
    pub fn opposed_distance_correction(
        self,
        multiplier: ConstraintMultiplier,
    ) -> PositionCorrection {
        PositionCorrection(-self.0 * multiplier.0)
    }
}
impl std::ops::Add for ParticleInverseMass {
    type Output = EffectiveInverseMass;
    fn add(self, other: Self) -> EffectiveInverseMass {
        EffectiveInverseMass(self.0 + other.0)
    }
}

/// Kilograms assigned to a mesh vertex, accumulated in triangle arrival order.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct ParticleMass(pub(super) f32);
impl From<f32> for ParticleMass {
    fn from(kilograms: f32) -> Self {
        Self(kilograms)
    }
}
impl ParticleMass {
    pub const ZERO: Self = Self(0.0);
    pub fn absolute(self) -> Self {
        Self(self.0.abs())
    }
    const MASSLESS_THRESHOLD_KG: f32 = 1e-12;
    pub fn validity(self) -> MassValidity {
        if self.0.is_finite() && self.0 >= 0.0 {
            MassValidity::FiniteNonnegative
        } else {
            MassValidity::Invalid
        }
    }
    pub fn inverse_mass(self) -> ParticleInverseMass {
        if self.0 > Self::MASSLESS_THRESHOLD_KG {
            ParticleInverseMass(1.0 / self.0)
        } else {
            ParticleInverseMass::PINNED
        }
    }
    fn native_sum_term(self) -> f32 {
        self.0
    }
}
impl std::ops::AddAssign for ParticleMass {
    fn add_assign(&mut self, share: Self) {
        self.0 += share.0;
    }
}
impl std::ops::Sub for ParticleMass {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}
impl std::fmt::Display for ParticleMass {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl std::iter::Sum for ParticleMass {
    fn sum<I: Iterator<Item = Self>>(masses: I) -> Self {
        Self(masses.map(Self::native_sum_term).sum())
    }
}
impl<'a> std::iter::Sum<&'a ParticleMass> for ParticleMass {
    fn sum<I: Iterator<Item = &'a ParticleMass>>(masses: I) -> Self {
        masses.copied().sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleArealDensity(f32);
impl From<f32> for ParticleArealDensity {
    fn from(kilograms_per_square_metre: f32) -> Self {
        Self(kilograms_per_square_metre)
    }
}
impl ParticleArealDensity {
    pub const fn kilograms_per_square_metre(value: f32) -> Self {
        Self(value)
    }
    pub fn validity(self) -> ArealDensityValidity {
        if self.0.is_finite() && self.0 > 0.0 {
            ArealDensityValidity::PositiveFinite
        } else {
            ArealDensityValidity::Invalid
        }
    }
    pub fn vertex_share(self, points: [Vec3; 3]) -> ParticleMass {
        let [a, b, c] = points;
        let area = (b - a).cross(c - a).length() * 0.5;
        ParticleMass(area * self.0 / 3.0)
    }
}

#[cfg(test)]
mod tests;
