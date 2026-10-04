//! Spatial admission for player chat after personal frontiers align.

use super::super::{
    AppState,
    party_actions::{CaseSiteObservationError, character_case_site_id},
};
use crate::spacetimedb::{CaseSiteId, CharacterView};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LocalPlayerPresence {
    CoLocated,
    Separate,
}

impl LocalPlayerPresence {
    pub(super) async fn observe(
        state: &AppState,
        actor: &CharacterView,
        subject: &CharacterView,
    ) -> std::result::Result<Self, CaseSiteObservationError> {
        let actor_site = character_case_site_id(state, actor.id.into()).await?;
        let subject_site = character_case_site_id(state, subject.id.into()).await?;
        Ok(Self::admit(actor, subject, actor_site, subject_site))
    }

    fn admit(
        actor: &CharacterView,
        subject: &CharacterView,
        actor_site: Option<CaseSiteId>,
        subject_site: Option<CaseSiteId>,
    ) -> Self {
        if actor.current_settlement_id != subject.current_settlement_id
            || actor_site != subject_site
            || (actor.current_settlement_id.is_none() && actor_site.is_none())
        {
            Self::Separate
        } else {
            Self::CoLocated
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::identity::CharacterId;

    fn character(id: CharacterId) -> CharacterView {
        CharacterView {
            id: id.into(),
            name: "Traveller".into(),
            xp: 0,
            level: 1,
            current_settlement_id: None,
            current_case_site_id: None,
            party_id: None,
            age_years: 25,
            alive: true,
            temporary: false,
            social_notification_count: 0,
            automatic_social_chat_enabled: false,
        }
    }

    #[test]
    fn unknown_locations_do_not_authorize_player_chat() {
        let actor = character(7.into());
        let subject = character(8.into());
        assert_eq!(
            LocalPlayerPresence::admit(&actor, &subject, None, None),
            LocalPlayerPresence::Separate
        );
        let site = CaseSiteId::try_new("case-site-1").unwrap();
        assert_eq!(
            LocalPlayerPresence::admit(&actor, &subject, Some(site.clone()), Some(site)),
            LocalPlayerPresence::CoLocated
        );
    }

    #[test]
    fn settlement_and_exact_site_authorities_must_both_agree() {
        let mut actor = character(7.into());
        let mut subject = character(8.into());
        actor.current_settlement_id = Some("riverdale".into());
        subject.current_settlement_id = actor.current_settlement_id.clone();
        assert_eq!(
            LocalPlayerPresence::admit(&actor, &subject, None, None),
            LocalPlayerPresence::CoLocated
        );
        let first = CaseSiteId::try_new("case-site-1").unwrap();
        let second = CaseSiteId::try_new("case-site-2").unwrap();
        assert_eq!(
            LocalPlayerPresence::admit(&actor, &subject, Some(first), Some(second)),
            LocalPlayerPresence::Separate
        );
        subject.current_settlement_id = Some("westhaven".into());
        assert_eq!(
            LocalPlayerPresence::admit(&actor, &subject, None, None),
            LocalPlayerPresence::Separate
        );
    }
}
