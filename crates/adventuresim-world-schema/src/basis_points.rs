//! Bounded fractions and their conversion to game quantities.
use serde::{Deserialize, Serialize};

/// The basis-point representation of one whole (100%).
pub const BASIS_POINTS_PER_WHOLE: u16 = 10_000;

/// A validated fraction of one whole, represented in basis points.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(try_from = "u16", into = "u16")]
pub struct UnitBasisPoints {
    basis_points: u16,
}

impl UnitBasisPoints {
    pub const ZERO: Self = Self { basis_points: 0 };
    pub const WHOLE: Self = Self {
        basis_points: BASIS_POINTS_PER_WHOLE,
    };

    pub const fn new(value: u16) -> Option<Self> {
        if value <= BASIS_POINTS_PER_WHOLE {
            Some(Self {
                basis_points: value,
            })
        } else {
            None
        }
    }

    pub const fn saturating(value: u16) -> Self {
        Self {
            basis_points: if value > BASIS_POINTS_PER_WHOLE {
                BASIS_POINTS_PER_WHOLE
            } else {
                value
            },
        }
    }

    pub fn from_unit_f32(value: f32) -> Option<Self> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return None;
        }
        Self::new((value * f32::from(BASIS_POINTS_PER_WHOLE)).round() as u16)
    }

    pub fn from_unit_f64(value: f64) -> Option<Self> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return None;
        }
        Self::new((value * f64::from(BASIS_POINTS_PER_WHOLE)).round() as u16)
    }

    pub fn from_ratio_floor(numerator: u64, denominator: u64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let value = u128::from(numerator).saturating_mul(u128::from(BASIS_POINTS_PER_WHOLE))
            / u128::from(denominator);
        Some(Self::saturating(value.min(u128::from(u16::MAX)) as u16))
    }

    pub const fn get(self) -> u16 {
        self.basis_points
    }

    pub const fn complement(self) -> Self {
        Self {
            basis_points: BASIS_POINTS_PER_WHOLE - self.basis_points,
        }
    }

    pub fn as_unit_f32(self) -> f32 {
        f32::from(self.basis_points) / f32::from(BASIS_POINTS_PER_WHOLE)
    }

    pub fn as_unit_f64(self) -> f64 {
        f64::from(self.basis_points) / f64::from(BASIS_POINTS_PER_WHOLE)
    }

    pub const fn scale_u32_floor(self, whole: u32) -> u32 {
        ((whole as u64 * self.basis_points as u64) / BASIS_POINTS_PER_WHOLE as u64) as u32
    }

    pub const fn scale_u64_floor(self, whole: u64) -> u64 {
        ((whole as u128 * self.basis_points as u128) / BASIS_POINTS_PER_WHOLE as u128) as u64
    }
}

/// A validated signed fraction of one whole, represented in basis points.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(try_from = "i16", into = "i16")]
pub struct SignedUnitBasisPoints {
    basis_points: i16,
}

/// A fraction outside the unsigned or signed unit interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BasisPointsOutOfRange;

impl std::fmt::Display for BasisPointsOutOfRange {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("basis points exceed the unit interval")
    }
}

impl std::error::Error for BasisPointsOutOfRange {}

macro_rules! checked_fraction {
    ($name:ident, $raw:ty) => {
        impl TryFrom<$raw> for $name {
            type Error = BasisPointsOutOfRange;

            fn try_from(value: $raw) -> Result<Self, Self::Error> {
                Self::new(value).ok_or(BasisPointsOutOfRange)
            }
        }

        impl From<$name> for $raw {
            fn from(value: $name) -> Self {
                value.get()
            }
        }

        #[cfg(feature = "spacetimedb")]
        crate::checked_sats::checked_numeric_product!($name, basis_points: $raw);
    };
}

checked_fraction!(UnitBasisPoints, u16);
checked_fraction!(SignedUnitBasisPoints, i16);

impl SignedUnitBasisPoints {
    pub const MIN: Self = Self {
        basis_points: -(BASIS_POINTS_PER_WHOLE as i16),
    };
    pub const MAX: Self = Self {
        basis_points: BASIS_POINTS_PER_WHOLE as i16,
    };
    pub const ZERO: Self = Self { basis_points: 0 };

    pub const fn new(value: i16) -> Option<Self> {
        if value >= Self::MIN.basis_points && value <= Self::MAX.basis_points {
            Some(Self {
                basis_points: value,
            })
        } else {
            None
        }
    }

    pub const fn get(self) -> i16 {
        self.basis_points
    }

    pub fn as_unit_f32(self) -> f32 {
        f32::from(self.basis_points) / f32::from(BASIS_POINTS_PER_WHOLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoding_rejects_invalid_fractions_and_preserves_valid_scalars() {
        for value in ["10001", "65535", "-1"] {
            assert!(serde_json::from_str::<UnitBasisPoints>(value).is_err());
        }
        for value in ["-10001", "10001", "32767", "-32768"] {
            assert!(serde_json::from_str::<SignedUnitBasisPoints>(value).is_err());
        }
        for value in ["0", "2500", "10000"] {
            let fraction: UnitBasisPoints = serde_json::from_str(value).unwrap();
            assert_eq!(serde_json::to_string(&fraction).unwrap(), value);
            assert!(fraction.complement().get() <= BASIS_POINTS_PER_WHOLE);
        }
        for value in ["-10000", "0", "10000"] {
            let fraction: SignedUnitBasisPoints = serde_json::from_str(value).unwrap();
            assert_eq!(serde_json::to_string(&fraction).unwrap(), value);
        }
    }

    #[cfg(feature = "spacetimedb")]
    #[test]
    fn database_decoding_enforces_the_same_fraction_bounds() {
        use spacetimedb_lib::bsatn;

        assert!(bsatn::from_slice::<UnitBasisPoints>(&10_001_u16.to_le_bytes()).is_err());
        let invalid = 10_001_u16.to_le_bytes();
        let mut input = invalid.as_slice();
        assert!(
            <UnitBasisPoints as spacetimedb_lib::de::Deserialize>::validate(
                bsatn::Deserializer::new(&mut input)
            )
            .is_err()
        );
        assert!(bsatn::from_slice::<SignedUnitBasisPoints>(&(-10_001_i16).to_le_bytes()).is_err());
        for value in [UnitBasisPoints::ZERO, UnitBasisPoints::WHOLE] {
            let bytes = bsatn::to_vec(&value).unwrap();
            assert_eq!(bytes, value.get().to_le_bytes());
            assert_eq!(bsatn::from_slice::<UnitBasisPoints>(&bytes).unwrap(), value);
        }
    }

    #[test]
    fn basis_point_units_own_bounds_conversions_and_scaling() {
        let quarter = UnitBasisPoints::from_unit_f32(0.25).unwrap();
        assert_eq!(quarter.get(), 2_500);
        assert_eq!(quarter.as_unit_f32(), 0.25);
        assert_eq!(quarter.complement().get(), 7_500);
        assert_eq!(quarter.scale_u32_floor(10), 2);
        assert_eq!(UnitBasisPoints::new(BASIS_POINTS_PER_WHOLE + 1), None);
        assert_eq!(
            SignedUnitBasisPoints::new(-10_000),
            Some(SignedUnitBasisPoints::MIN)
        );
        assert!(SignedUnitBasisPoints::new(-10_001).is_none());
    }
}
