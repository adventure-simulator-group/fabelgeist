//! Skeletal body proportions, independent of surface identity and animated pose.

use serde::{Deserialize, Serialize};

pub const BODY_PROPORTION_COUNT: usize = 9;
const GENERATED_RANGE_FRACTION: f32 = 0.35;
const SKELETAL_SEED_DOMAIN: u64 = 0x534b_454c_4554_414c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyProportion {
    HipWidth,
    ShoulderWidth,
    UpperArmLength,
    LowerArmLength,
    UpperLegLength,
    LowerLegLength,
    SpineLength,
    NeckLength,
    FootLength,
}

impl BodyProportion {
    pub const ALL: [Self; BODY_PROPORTION_COUNT] = [
        Self::HipWidth,
        Self::ShoulderWidth,
        Self::UpperArmLength,
        Self::LowerArmLength,
        Self::UpperLegLength,
        Self::LowerLegLength,
        Self::SpineLength,
        Self::NeckLength,
        Self::FootLength,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::HipWidth => "Hip width",
            Self::ShoulderWidth => "Shoulder width",
            Self::UpperArmLength => "Upper arm length",
            Self::LowerArmLength => "Lower arm length",
            Self::UpperLegLength => "Upper leg length",
            Self::LowerLegLength => "Lower leg length",
            Self::SpineLength => "Spine length",
            Self::NeckLength => "Neck length",
            Self::FootLength => "Foot length",
        }
    }

    pub fn mhr_parameter(self) -> &'static str {
        match self {
            Self::HipWidth => "scale_hip_width",
            Self::ShoulderWidth => "scale_shoulder_width",
            Self::UpperArmLength => "scale_uparms",
            Self::LowerArmLength => "scale_lowarms",
            Self::UpperLegLength => "scale_uplegs",
            Self::LowerLegLength => "scale_lowlegs",
            Self::SpineLength => "scale_spine_length",
            Self::NeckLength => "scale_neck_length",
            Self::FootLength => "scale_foot_length",
        }
    }

    /// Symmetric limits from the pinned MHR v1.0.1 model definition.
    pub fn limit(self) -> f32 {
        match self {
            Self::HipWidth | Self::UpperLegLength => 0.5,
            Self::ShoulderWidth => 0.2,
            Self::UpperArmLength | Self::LowerArmLength | Self::LowerLegLength => 1.0,
            Self::SpineLength => 1.1,
            Self::NeckLength => 0.4,
            Self::FootLength => 0.1,
        }
    }
}

/// Absolute MHR skeletal coefficients. Zero is the reference skeleton.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "[f32; BODY_PROPORTION_COUNT]",
    into = "[f32; BODY_PROPORTION_COUNT]"
)]
pub struct CharacterProportions([f32; BODY_PROPORTION_COUNT]);

impl CharacterProportions {
    pub fn get(self, proportion: BodyProportion) -> f32 {
        self.0[proportion.index()]
    }

    pub fn set(&mut self, proportion: BodyProportion, value: f32) -> Result<(), &'static str> {
        if !value.is_finite() || value.abs() > proportion.limit() {
            return Err("skeletal proportion is outside its finite MHR limits");
        }
        self.0[proportion.index()] = value;
        Ok(())
    }

    /// Stable cosmetic variation; does not alter tactical physics or reach.
    pub fn from_character_id(id: u64) -> Self {
        let mut random = fabelgeist_determinism::SplitMix64::new(id ^ SKELETAL_SEED_DOMAIN);
        Self(std::array::from_fn(|index| {
            let unit = fabelgeist_determinism::inclusive_unit_f32(random.next_u64());
            (unit * 2.0 - 1.0) * BodyProportion::ALL[index].limit() * GENERATED_RANGE_FRACTION
        }))
    }
}

impl TryFrom<[f32; BODY_PROPORTION_COUNT]> for CharacterProportions {
    type Error = &'static str;
    fn try_from(values: [f32; BODY_PROPORTION_COUNT]) -> Result<Self, Self::Error> {
        let mut result = Self::default();
        for proportion in BodyProportion::ALL {
            result.set(proportion, values[proportion.index()])?;
        }
        Ok(result)
    }
}

impl From<CharacterProportions> for [f32; BODY_PROPORTION_COUNT] {
    fn from(value: CharacterProportions) -> Self {
        value.0
    }
}

/// Per-joint glTF metadata. Deltas are local translations in metres per unit
/// coefficient. The supported MHR controls do not change local rotation/scale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointProportionBasis {
    pub reference: CharacterProportions,
    pub translation_metres: [[f32; 3]; BODY_PROPORTION_COUNT],
}

impl JointProportionBasis {
    pub fn translation(&self, proportions: CharacterProportions) -> [f32; 3] {
        std::array::from_fn(|axis| {
            BodyProportion::ALL
                .iter()
                .map(|p| {
                    self.translation_metres[p.index()][axis]
                        * (proportions.get(*p) - self.reference.get(*p))
                })
                .sum()
        })
    }

    pub fn is_finite(&self) -> bool {
        self.translation_metres
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeletal_proportions_are_stable_bounded_and_validate_serialization() {
        let first = CharacterProportions::from_character_id(42);
        assert_eq!(first, CharacterProportions::from_character_id(42));
        assert_ne!(first, CharacterProportions::from_character_id(43));
        assert_eq!(
            first,
            serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap()
        );
        for p in BodyProportion::ALL {
            assert!(first.get(p).abs() <= p.limit());
            assert!(CharacterProportions::default().set(p, f32::NAN).is_err());
            let mut invalid = [0.0; BODY_PROPORTION_COUNT];
            invalid[p.index()] = p.limit() + 1.0;
            assert!(
                serde_json::from_str::<CharacterProportions>(
                    &serde_json::to_string(&invalid).unwrap()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn skeletal_basis_is_relative_to_exported_recipe() {
        let mut reference = CharacterProportions::default();
        reference.set(BodyProportion::HipWidth, 0.2).unwrap();
        let mut basis = JointProportionBasis {
            reference,
            translation_metres: [[0.0; 3]; BODY_PROPORTION_COUNT],
        };
        basis.translation_metres[BodyProportion::HipWidth.index()] = [0.1, 0.0, 0.0];
        assert_eq!(basis.translation(reference), [0.0; 3]);
        assert!((basis.translation(CharacterProportions::default())[0] + 0.02).abs() < 1e-6);
    }
}
