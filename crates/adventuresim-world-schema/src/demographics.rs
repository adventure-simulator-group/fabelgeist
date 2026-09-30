//! Shared demographic vocabulary for strategic identity and naming.

use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Sex {
    Female,
    Male,
}

impl Sex {
    /// Variants available for uniform deterministic selection.
    pub const VARIANTS: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    pub fn stable_id(self) -> &'static str {
        self.into()
    }

    /// Historical variant spelling used in persisted deterministic seed inputs.
    pub const fn stable_variant_id(self) -> &'static str {
        match self {
            Self::Female => "Female",
            Self::Male => "Male",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
#[serde(rename_all = "snake_case")]
pub enum Culture {
    German,
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_determinism::{DeterministicRng, Seed};

    #[test]
    fn demographic_values_roundtrip_with_stable_catalog_ids() {
        for (sex, variant_id) in [(Sex::Female, "Female"), (Sex::Male, "Male")] {
            let json = serde_json::to_string(&sex).unwrap();
            assert_eq!(json, format!("\"{}\"", sex.stable_id()));
            assert_eq!(serde_json::from_str::<Sex>(&json).unwrap(), sex);
            assert_eq!(sex.stable_id().parse::<Sex>().unwrap(), sex);
            assert_eq!(sex.stable_variant_id(), variant_id);
        }
        assert_eq!(
            serde_json::to_string(&Culture::German).unwrap(),
            "\"german\""
        );
        assert_eq!(
            serde_json::from_str::<Culture>("\"german\"").unwrap(),
            Culture::German
        );
        assert!(serde_json::from_str::<Sex>("\"unknown\"").is_err());
        assert!("Female".parse::<Sex>().is_err());
        assert!(serde_json::from_str::<Culture>("\"english\"").is_err());
    }

    #[test]
    fn typed_sex_draw_preserves_the_existing_binary_draw() {
        for seed in [0, 1, 17, 42, u64::MAX] {
            let mut previous = DeterministicRng::new(Seed::from_u64(seed));
            let mut typed = previous.clone();
            let expected = if previous.boolean() {
                Sex::Female
            } else {
                Sex::Male
            };
            assert_eq!(*typed.choose(Sex::VARIANTS), expected);
        }
    }
}
