//! Capture validation and consumption of immutable mission enemy IDs.
#[cfg(feature = "authority-tests")]
mod authority_tests;
use super::{CaseSiteAuthority, HostileGroupAuthority, MissionAttemptStatus, MissionAuthority};
use crate::character::character;
use adventuresim_core::mission::{EnemyRosterError, MissionEnemyRoster};
use spacetimedb::ReducerContext;
use std::fmt;

#[derive(Debug)]
pub(crate) enum MissionRosterError {
    Invalid(EnemyRosterError),
    GroupCountMismatch,
    MissingCharacter(u64),
}
impl From<EnemyRosterError> for MissionRosterError {
    fn from(error: EnemyRosterError) -> Self {
        Self::Invalid(error)
    }
}
impl fmt::Display for MissionRosterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(error) => error.fmt(formatter),
            Self::GroupCountMismatch => {
                formatter.write_str("Hostile group roster does not match group count")
            }
            Self::MissingCharacter(id) => {
                write!(formatter, "Captured mission enemy {id} is missing")
            }
        }
    }
}
impl std::error::Error for MissionRosterError {}

impl HostileGroupAuthority {
    /// Check live planning cardinality once, before storing the exact roster.
    pub(crate) fn capture_enemy_roster(
        &self,
        ctx: &ReducerContext,
    ) -> Result<MissionEnemyRoster, MissionRosterError> {
        let roster =
            MissionEnemyRoster::try_from(crate::world_actor::context_character_ids(ctx, &self.id))?;
        if roster.enemy_count().get() != self.enemy_count {
            return Err(MissionRosterError::GroupCountMismatch);
        }
        Ok(roster)
    }
}

impl MissionAuthority {
    /// Capture binding state and all combat/loot inputs under one constructor.
    pub(crate) fn capture(
        ctx: &ReducerContext,
        id: &str,
        party_id: &str,
        observer_character_id: u64,
        site: &CaseSiteAuthority,
        group: &HostileGroupAuthority,
        scene_key: &str,
    ) -> Result<Self, MissionRosterError> {
        Ok(Self {
            id: id.into(),
            party_id: party_id.into(),
            case_site_id: Some(site.id.clone()),
            hostile_group_id: Some(group.id.clone()),
            observer_character_id,
            case_id: site.case_id.clone(),
            outcome_entropy: ctx.random(),
            status: MissionAttemptStatus::Bound,
            committed_resolution: None,
            committed_capture_subject_id: None,
            committed_capture_custody_version: None,
            scene_key: scene_key.into(),
            hostile_version: group.escalation_incident_ordinal,
            enemy_character_ids: group.capture_enemy_roster(ctx)?.into_enemy_ids(),
            contacted_before_combat: crate::world_actor::party_contacted_context(
                ctx, party_id, &group.id,
            ),
            enemy_difficulty: group.base_difficulty,
            enemy_combat_scale_bps: group.combat_scale_bps,
            normalized_combat_power: group.normalized_combat_power,
            drop_item_id: group.drop_item_id.clone(),
            drop_quantity: group.drop_quantity,
        })
    }

    /// Validate the stored snapshot without rereading a live group's size.
    pub(crate) fn enemy_roster(
        &self,
        ctx: &ReducerContext,
    ) -> Result<MissionEnemyRoster, MissionRosterError> {
        let roster = MissionEnemyRoster::try_from(self.enemy_character_ids.clone())?;
        for id in roster.enemy_ids() {
            if ctx.db.character().id().find(*id).is_none() {
                return Err(MissionRosterError::MissingCharacter(*id));
            }
        }
        Ok(roster)
    }
}
