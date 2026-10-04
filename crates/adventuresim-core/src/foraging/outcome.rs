//! Resolution evidence and its player-safe legality projection.

use super::ForageYieldQuantity;
use crate::item_catalog::ItemDefinitionId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForageYield {
    pub item_id: ItemDefinitionId,
    pub quantity: ForageYieldQuantity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForageResolution {
    pub yields: Vec<ForageYield>,
    pub exposure: ForageExposure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForageStealthOutcome {
    NotChecked,
    Concealed,
    Detected,
}

impl From<Option<bool>> for ForageStealthOutcome {
    fn from(value: Option<bool>) -> Self {
        match value {
            None => Self::NotChecked,
            Some(true) => Self::Concealed,
            Some(false) => Self::Detected,
        }
    }
}

/// Unchecked exposure has no difficulty; a completed check always retains its
/// difficulty and one of the two checked outcomes. Fields cannot be mixed by
/// consumers. Only native receipt serialization exposes their representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForageExposure {
    difficulty_millirank: Option<u16>,
    outcome: ForageStealthOutcome,
}

impl ForageExposure {
    pub const UNCHECKED: Self = Self {
        difficulty_millirank: None,
        outcome: ForageStealthOutcome::NotChecked,
    };

    pub(super) const fn checked(difficulty_millirank: u16, concealed: bool) -> Self {
        Self {
            difficulty_millirank: Some(difficulty_millirank),
            outcome: if concealed {
                ForageStealthOutcome::Concealed
            } else {
                ForageStealthOutcome::Detected
            },
        }
    }

    pub const fn outcome(self) -> ForageStealthOutcome {
        self.outcome
    }
}

impl From<ForageExposure> for (Option<u16>, Option<bool>) {
    fn from(value: ForageExposure) -> Self {
        let succeeded = match value.outcome {
            ForageStealthOutcome::NotChecked => None,
            ForageStealthOutcome::Concealed => Some(true),
            ForageStealthOutcome::Detected => Some(false),
        };
        (value.difficulty_millirank, succeeded)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForageLegality {
    Legal,
    IllegalAttempt,
}

impl From<bool> for ForageLegality {
    fn from(illegal: bool) -> Self {
        if illegal {
            Self::IllegalAttempt
        } else {
            Self::Legal
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForagePublicLegalOutcome {
    Legal,
    Unnoticed,
    Noticed,
}

impl ForagePublicLegalOutcome {
    /// Preserve the public receipt policy, including its noticed wording for
    /// an illegal attempt whose private exposure check was not performed.
    pub const fn from_authority(legality: ForageLegality, stealth: ForageStealthOutcome) -> Self {
        match (legality, stealth) {
            (ForageLegality::Legal, _) => Self::Legal,
            (ForageLegality::IllegalAttempt, ForageStealthOutcome::Concealed) => Self::Unnoticed,
            (ForageLegality::IllegalAttempt, _) => Self::Noticed,
        }
    }
}

impl std::fmt::Display for ForagePublicLegalOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Legal => "legal",
            Self::Unnoticed => "unnoticed",
            Self::Noticed => "noticed",
        })
    }
}

impl TryFrom<&str> for ForagePublicLegalOutcome {
    type Error = ForageLegalOutcomeError;

    fn try_from(encoded: &str) -> Result<Self, Self::Error> {
        match encoded {
            "legal" => Ok(Self::Legal),
            "unnoticed" => Ok(Self::Unnoticed),
            "noticed" => Ok(Self::Noticed),
            _ => Err(ForageLegalOutcomeError {
                encoded: encoded.into(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForageLegalOutcomeError {
    encoded: String,
}

impl std::fmt::Display for ForageLegalOutcomeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Unknown public forage legal outcome: {}",
            self.encoded
        )
    }
}

impl std::error::Error for ForageLegalOutcomeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_exposure_encodings_and_public_wording_keep_distinct_policies() {
        for (exposure, encoding) in [
            (ForageExposure::UNCHECKED, (None, None)),
            (
                ForageExposure::checked(1_750, true),
                (Some(1_750), Some(true)),
            ),
            (
                ForageExposure::checked(4_225, false),
                (Some(4_225), Some(false)),
            ),
        ] {
            assert_eq!(<(Option<u16>, Option<bool>)>::from(exposure), encoding);
            assert_eq!(ForageStealthOutcome::from(encoding.1), exposure.outcome());
            assert_eq!(
                ForagePublicLegalOutcome::from_authority(ForageLegality::Legal, exposure.outcome()),
                ForagePublicLegalOutcome::Legal
            );
            let expected = match exposure.outcome() {
                ForageStealthOutcome::Concealed => ForagePublicLegalOutcome::Unnoticed,
                ForageStealthOutcome::NotChecked | ForageStealthOutcome::Detected => {
                    ForagePublicLegalOutcome::Noticed
                }
            };
            assert_eq!(
                ForagePublicLegalOutcome::from_authority(
                    ForageLegality::IllegalAttempt,
                    exposure.outcome()
                ),
                expected
            );
        }
    }

    #[test]
    fn legal_receipt_tokens_are_exact_and_unknown_spelling_is_retained() {
        for (encoded, outcome) in [
            ("legal", ForagePublicLegalOutcome::Legal),
            ("unnoticed", ForagePublicLegalOutcome::Unnoticed),
            ("noticed", ForagePublicLegalOutcome::Noticed),
        ] {
            assert_eq!(
                ForagePublicLegalOutcome::try_from(encoded).unwrap(),
                outcome
            );
            assert_eq!(outcome.to_string(), encoded);
            assert_eq!(
                serde_json::to_value(outcome).unwrap(),
                serde_json::json!(encoded)
            );
            assert_eq!(
                serde_json::from_value::<ForagePublicLegalOutcome>(serde_json::json!(encoded))
                    .unwrap(),
                outcome
            );
        }
        for encoded in ["", "lawful", "Noticed", "noticed ", "unnoticed\n"] {
            let error = ForagePublicLegalOutcome::try_from(encoded).unwrap_err();
            assert_eq!(error.encoded, encoded);
            assert!(error.to_string().contains(encoded));
            assert!(
                serde_json::from_value::<ForagePublicLegalOutcome>(serde_json::json!(encoded))
                    .is_err()
            );
        }
    }
}
