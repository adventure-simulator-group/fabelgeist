//! Diagnostic ordering of anatomical crest query directions.

use crate::breastplate_shoulder_band::{BandError, BandFraction};
use serde::Serialize;
use std::ffi::OsStr;

const SUPERIOR_FIRST_ENV: &str = "BREASTPLATE_DIAGNOSTIC_SUPERIOR_FIRST";
const ANTERIOR_RATIOS: &[f32] = &[0.25, 0.5, 0.75, 1.0, 1.5, 2.0];
const SUPERIOR_RATIOS: &[f32] = &[0.0, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0];

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CrestQueryPolicy {
    AnteriorFirst,
    SuperiorFirst,
}

impl CrestQueryPolicy {
    pub(crate) fn from_environment(band: Option<BandFraction>) -> Result<Self, BandError> {
        Self::parse(std::env::var_os(SUPERIOR_FIRST_ENV).as_deref(), band)
    }

    fn parse(value: Option<&OsStr>, band: Option<BandFraction>) -> Result<Self, BandError> {
        if value.is_some() {
            if band.is_none() {
                return Err(BandError::RequiresShoulderBand);
            }
            Ok(Self::SuperiorFirst)
        } else {
            Ok(Self::AnteriorFirst)
        }
    }

    pub(crate) fn ratios(self) -> &'static [f32] {
        match self {
            Self::AnteriorFirst => ANTERIOR_RATIOS,
            Self::SuperiorFirst => SUPERIOR_RATIOS,
        }
    }

    pub(crate) fn hash(self, hash: &mut blake3::Hasher) {
        if self == Self::SuperiorFirst {
            hash.update(b"superior-first-crest-v1");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_in_adds_only_superior_candidate_and_requires_band() {
        let band = Some(BandFraction::new(0.3).unwrap());
        let enabled = CrestQueryPolicy::parse(Some(OsStr::new("1")), band).unwrap();
        assert_eq!(enabled.ratios()[0], 0.0);
        assert_eq!(
            &enabled.ratios()[1..],
            CrestQueryPolicy::AnteriorFirst.ratios()
        );
        assert!(matches!(
            CrestQueryPolicy::parse(Some(OsStr::new("1")), None),
            Err(BandError::RequiresShoulderBand)
        ));
        assert_eq!(
            CrestQueryPolicy::parse(None, band).unwrap(),
            CrestQueryPolicy::AnteriorFirst
        );
    }

    #[test]
    fn default_cache_is_unchanged_and_opt_in_is_distinct() {
        let mut baseline = blake3::Hasher::new();
        baseline.update(b"baseline field key");
        let expected = baseline.finalize();
        CrestQueryPolicy::AnteriorFirst.hash(&mut baseline);
        assert_eq!(baseline.finalize(), expected);
        CrestQueryPolicy::SuperiorFirst.hash(&mut baseline);
        assert_ne!(baseline.finalize(), expected);
    }
}
