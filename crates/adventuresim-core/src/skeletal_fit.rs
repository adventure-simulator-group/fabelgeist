//! Residual equipment fitting after skeletal deformation.

use crate::character_proportions::{BodyProportion, CharacterProportions};

#[derive(Clone, Copy)]
pub enum SkeletalFitMorph {
    ShortSpine,
    LongSpine,
    ShortNeck,
    LongNeck,
    ShortUpperArm,
    LongUpperArm,
    ShortUpperLeg,
    LongUpperLeg,
    NarrowHips,
    WideHips,
    ShortLowerLeg,
    LongLowerLeg,
}

impl SkeletalFitMorph {
    pub const ALL: [Self; 12] = [
        Self::ShortSpine,
        Self::LongSpine,
        Self::ShortNeck,
        Self::LongNeck,
        Self::ShortUpperArm,
        Self::LongUpperArm,
        Self::ShortUpperLeg,
        Self::LongUpperLeg,
        Self::NarrowHips,
        Self::WideHips,
        Self::ShortLowerLeg,
        Self::LongLowerLeg,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::ShortSpine => "mhr_skeletal_spine_short",
            Self::LongSpine => "mhr_skeletal_spine_long",
            Self::ShortNeck => "mhr_skeletal_neck_short",
            Self::LongNeck => "mhr_skeletal_neck_long",
            Self::ShortUpperArm => "mhr_skeletal_upper_arm_short",
            Self::LongUpperArm => "mhr_skeletal_upper_arm_long",
            Self::ShortUpperLeg => "mhr_skeletal_upper_leg_short",
            Self::LongUpperLeg => "mhr_skeletal_upper_leg_long",
            Self::NarrowHips => "mhr_skeletal_hip_narrow",
            Self::WideHips => "mhr_skeletal_hip_wide",
            Self::ShortLowerLeg => "mhr_skeletal_lower_leg_short",
            Self::LongLowerLeg => "mhr_skeletal_lower_leg_long",
        }
    }

    pub fn endpoint(self) -> f32 {
        match self {
            Self::ShortSpine
            | Self::ShortNeck
            | Self::ShortUpperArm
            | Self::ShortUpperLeg
            | Self::NarrowHips
            | Self::ShortLowerLeg => -self.proportion().limit(),
            Self::LongSpine
            | Self::LongNeck
            | Self::LongUpperArm
            | Self::LongUpperLeg
            | Self::WideHips
            | Self::LongLowerLeg => self.proportion().limit(),
        }
    }

    pub fn proportion(self) -> BodyProportion {
        match self {
            Self::ShortSpine | Self::LongSpine => BodyProportion::SpineLength,
            Self::ShortNeck | Self::LongNeck => BodyProportion::NeckLength,
            Self::ShortUpperArm | Self::LongUpperArm => BodyProportion::UpperArmLength,
            Self::ShortUpperLeg | Self::LongUpperLeg => BodyProportion::UpperLegLength,
            Self::NarrowHips | Self::WideHips => BodyProportion::HipWidth,
            Self::ShortLowerLeg | Self::LongLowerLeg => BodyProportion::LowerLegLength,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.name() == name)
    }

    pub fn weight(self, target: CharacterProportions, reference: CharacterProportions) -> f32 {
        let reference = reference.get(self.proportion());
        let interval = self.endpoint() - reference;
        if interval.abs() < f32::EPSILON {
            return 0.0;
        }
        ((target.get(self.proportion()) - reference) / interval).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeletal_fit_weights_are_relative_to_exported_body() {
        let mut reference = CharacterProportions::default();
        reference.set(BodyProportion::SpineLength, 0.4).unwrap();
        reference.set(BodyProportion::NeckLength, 0.1).unwrap();
        for morph in SkeletalFitMorph::ALL {
            assert_eq!(morph.weight(reference, reference), 0.0);
            let mut target = reference;
            target.set(morph.proportion(), morph.endpoint()).unwrap();
            assert_eq!(morph.weight(target, reference), 1.0);
            let mut middle = reference;
            middle
                .set(
                    morph.proportion(),
                    (reference.get(morph.proportion()) + morph.endpoint()) / 2.0,
                )
                .unwrap();
            assert!((morph.weight(middle, reference) - 0.5).abs() < 1e-6);
            assert_eq!(morph.weight(target, target), 0.0);
        }
    }

    #[test]
    fn neck_fitting_does_not_apply_a_spine_correction() {
        let reference = CharacterProportions::default();
        let mut target = reference;
        target.set(BodyProportion::NeckLength, -0.2).unwrap();
        assert_eq!(SkeletalFitMorph::ShortSpine.weight(target, reference), 0.0);
        assert_eq!(SkeletalFitMorph::LongSpine.weight(target, reference), 0.0);
        assert_eq!(SkeletalFitMorph::LongNeck.weight(target, reference), 0.0);
        assert!((SkeletalFitMorph::ShortNeck.weight(target, reference) - 0.5).abs() < 1e-6);
    }
}
