//! Checked starting-roster boundary values.
use super::{GENERATOR_VERSION, StartingAgeTier, StartingCharacterError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CandidateSeed(String);
impl CandidateSeed {
    pub fn try_new(seed: impl Into<String>) -> Result<Self, StartingCharacterError> {
        let seed = seed.into();
        if seed.len() != 32
            || !seed
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(StartingCharacterError::CandidateSeed);
        }
        Ok(Self(seed))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for CandidateSeed {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::try_new(String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// A slot whose bound was checked against the request's age tier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateSlot(u8);
impl CandidateSlot {
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingCharacterRequest {
    pub(super) seed: CandidateSeed,
    pub(super) slot: CandidateSlot,
    pub(super) age_tier: StartingAgeTier,
}
impl StartingCharacterRequest {
    pub fn parse(
        version: u16,
        seed: &str,
        age_tier: StartingAgeTier,
        slot: u8,
    ) -> Result<Self, StartingCharacterError> {
        if version != GENERATOR_VERSION {
            return Err(StartingCharacterError::GeneratorVersion);
        }
        let seed = CandidateSeed::try_new(seed)?;
        if slot >= age_tier.roster_size() {
            return Err(StartingCharacterError::CandidateSlot);
        }
        Ok(Self {
            seed,
            slot: CandidateSlot(slot),
            age_tier,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seed_decoding_and_request_rejection_share_the_seed_contract() {
        for invalid in ["", "f", "0000000000000000000000000000000F"] {
            let json = serde_json::to_string(invalid).unwrap();
            assert!(serde_json::from_str::<CandidateSeed>(&json).is_err());
            assert_eq!(
                StartingCharacterRequest::parse(
                    GENERATOR_VERSION,
                    invalid,
                    StartingAgeTier::Young,
                    0
                ),
                Err(StartingCharacterError::CandidateSeed)
            );
        }
        let valid = "00000000000000000000000000000000";
        assert_eq!(
            StartingCharacterRequest::parse(
                GENERATOR_VERSION,
                valid,
                StartingAgeTier::Young,
                u8::MAX
            ),
            Err(StartingCharacterError::CandidateSlot)
        );
        assert_eq!(
            StartingCharacterRequest::parse(
                GENERATOR_VERSION + 1,
                valid,
                StartingAgeTier::Young,
                0
            ),
            Err(StartingCharacterError::GeneratorVersion)
        );
    }
}
