//! Residual equipment fitting after skeletal deformation.

use crate::character_proportions::{BodyProportion, CharacterProportions};

#[derive(Clone, Copy)]
pub enum SkeletalFitMorph {
    ShortSpine,
    LongSpine,
}

impl SkeletalFitMorph {
    pub const ALL: [Self; 2] = [Self::ShortSpine, Self::LongSpine];

    pub fn name(self) -> &'static str {
        match self {
            Self::ShortSpine => "mhr_skeletal_spine_short",
            Self::LongSpine => "mhr_skeletal_spine_long",
        }
    }

    pub fn endpoint(self) -> f32 {
        match self {
            Self::ShortSpine => -BodyProportion::SpineLength.limit(),
            Self::LongSpine => BodyProportion::SpineLength.limit(),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.name() == name)
    }

    pub fn weight(self, target: CharacterProportions, reference: CharacterProportions) -> f32 {
        let reference = reference.get(BodyProportion::SpineLength);
        let interval = self.endpoint() - reference;
        if interval.abs() < f32::EPSILON {
            return 0.0;
        }
        ((target.get(BodyProportion::SpineLength) - reference) / interval).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeletal_fit_weights_are_relative_to_exported_body() {
        let mut reference = CharacterProportions::default();
        reference.set(BodyProportion::SpineLength, 0.4).unwrap();
        for morph in SkeletalFitMorph::ALL {
            assert_eq!(morph.weight(reference, reference), 0.0);
            let mut target = reference;
            target
                .set(BodyProportion::SpineLength, morph.endpoint())
                .unwrap();
            assert_eq!(morph.weight(target, reference), 1.0);
            let mut middle = reference;
            middle
                .set(BodyProportion::SpineLength, (0.4 + morph.endpoint()) / 2.0)
                .unwrap();
            assert!((morph.weight(middle, reference) - 0.5).abs() < 1e-6);
            assert_eq!(morph.weight(target, target), 0.0);
        }
    }
}
