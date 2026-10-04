//! Authorization failures retain private diagnostics until HTTP presentation.

use super::super::party_actions::CaseSiteObservationError;
use crate::spacetimedb::SpacetimeError;
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ChatReadStage {
    Actor,
    Settlement,
    Resident,
    Presence,
    ActorClock,
    ObservedPlayer,
    Frontier,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ChatAuthorizationError {
    #[error("{source}")]
    Database {
        stage: ChatReadStage,
        actor: CharacterId,
        subject: Option<CharacterId>,
        #[source]
        source: SpacetimeError,
    },
    #[error("Character not found")]
    MissingActor(CharacterId),
    #[error("Character has no party")]
    ActorHasNoParty(CharacterId),
    #[error("NPC is not local")]
    NpcNotLocal,
    #[error("NPC is not local")]
    InvalidNpcSubject(#[source] std::num::ParseIntError),
    #[error("Player conversations do not accept an NPC location")]
    UnexpectedPlayerLocation,
    #[error("Invalid player")]
    InvalidPlayerSubject(#[source] std::num::ParseIntError),
    #[error("Player is not available at your personal date")]
    PlayerUnavailable(CharacterId),
    #[error("Player is not at this location")]
    PlayerElsewhere(CharacterId),
    #[error("Player has no party")]
    PlayerHasNoParty(CharacterId),
    #[error("Unknown Local subject")]
    UnknownSubject,
    #[error("{0}")]
    CaseSite(#[from] CaseSiteObservationError),
}

impl ChatAuthorizationError {
    pub(super) fn database(
        stage: ChatReadStage,
        actor: CharacterId,
        subject: Option<CharacterId>,
        source: SpacetimeError,
    ) -> Self {
        Self::Database {
            stage,
            actor,
            subject,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn resident_read_keeps_actor_subject_stage_and_decoder_cause() {
        let decode = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let error = ChatAuthorizationError::database(
            ChatReadStage::Resident,
            7.into(),
            Some(41.into()),
            SpacetimeError::QueryResponseDecode(decode),
        );
        let ChatAuthorizationError::Database {
            stage,
            actor,
            subject,
            ..
        } = &error
        else {
            panic!("resident read must retain its authority context");
        };
        assert_eq!(*stage, ChatReadStage::Resident);
        assert_eq!(*actor, CharacterId::from(7));
        assert_eq!(*subject, Some(CharacterId::from(41)));
        let query = error
            .source()
            .unwrap()
            .downcast_ref::<SpacetimeError>()
            .unwrap();
        assert!(query.source().unwrap().is::<serde_json::Error>());
    }

    #[test]
    fn invalid_subjects_retain_parse_causes_and_established_notices() {
        let invalid = "-1".parse::<u64>().unwrap_err();
        let npc = ChatAuthorizationError::InvalidNpcSubject(invalid.clone());
        let player = ChatAuthorizationError::InvalidPlayerSubject(invalid);
        assert_eq!(npc.to_string(), "NPC is not local");
        assert_eq!(player.to_string(), "Invalid player");
        assert!(npc.source().unwrap().is::<std::num::ParseIntError>());
        assert!(player.source().unwrap().is::<std::num::ParseIntError>());
    }
}
