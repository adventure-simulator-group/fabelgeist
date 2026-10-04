//! Public rest admission keeps life, location and service refusals distinct.

use crate::character::LivingCharacterError;
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::SettlementActionService;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RestServiceAdmissionError {
    Living(LivingCharacterError),
    NotAtSettlement { character: CharacterId },
    MissingSettlement { character: CharacterId },
    UnavailableService { service: SettlementActionService },
}

impl From<LivingCharacterError> for RestServiceAdmissionError {
    fn from(source: LivingCharacterError) -> Self {
        Self::Living(source)
    }
}

impl std::fmt::Display for RestServiceAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Living(source) => source.fmt(f),
            Self::NotAtSettlement { .. } => {
                f.write_str("Settlement rest requires the character to be at a settlement")
            }
            Self::MissingSettlement { .. } => f.write_str("Character's settlement not found"),
            Self::UnavailableService { .. } => {
                f.write_str("This settlement does not offer the requested rest service")
            }
        }
    }
}

impl std::error::Error for RestServiceAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Living(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn public_rest_preserves_missing_and_dead_classification_and_character() {
        let missing = LivingCharacterError::Missing {
            character: CharacterId::from(7),
        };
        let dead = LivingCharacterError::Dead {
            character: CharacterId::from(17),
        };
        for (source, expected) in [
            (missing, "Character not found"),
            (dead, "Dead characters cannot perform this action"),
        ] {
            let error = RestServiceAdmissionError::from(source);
            assert_eq!(error, RestServiceAdmissionError::Living(source));
            assert_eq!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<LivingCharacterError>(),
                Some(&source)
            );
            assert_eq!(error.to_string(), expected);
        }
    }
}
