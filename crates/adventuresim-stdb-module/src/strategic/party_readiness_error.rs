//! Whole-party action admission preserves concrete participant failures.

use crate::condition::CharacterReadinessError;

#[derive(Debug)]
pub(crate) enum PartyReadinessError {
    NoLivingMembers,
    Character(CharacterReadinessError),
}

impl From<CharacterReadinessError> for PartyReadinessError {
    fn from(source: CharacterReadinessError) -> Self {
        Self::Character(source)
    }
}

impl std::fmt::Display for PartyReadinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLivingMembers => f.write_str("Party has no living members"),
            Self::Character(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for PartyReadinessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoLivingMembers => None,
            Self::Character(source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::readiness_error::ReadinessParticipant;
    use adventuresim_core::identity::CharacterId;
    use std::error::Error;

    #[test]
    fn party_admission_preserves_incapacitated_member_identity_and_role() {
        let member = CharacterId::from(17);
        let error = PartyReadinessError::from(CharacterReadinessError::Incapacitated(
            ReadinessParticipant::PartyMember(member),
        ));
        assert!(
            matches!(error.source().unwrap().downcast_ref::<CharacterReadinessError>(), Some(CharacterReadinessError::Incapacitated(ReadinessParticipant::PartyMember(actual))) if *actual == member)
        );
        assert_eq!(
            error.to_string(),
            "A party member is incapacitated and must recover before acting"
        );
    }
}
