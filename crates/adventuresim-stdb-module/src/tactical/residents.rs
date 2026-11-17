//! An immutable area roster exposes only selected residents to its scene host.

use std::collections::BTreeSet;

use adventuresim_core::{
    strategic_place::{SettlementVenueKind, StrategicPlaceId},
    tactical_session::MAX_LOCAL_SCENE_RESIDENTS,
};

use super::*;
use crate::relationship::{npc_policy, npc_policy__view};
use crate::settlement_population::resident_drafts::business_location;
use crate::settlement_population::{
    SettlementResidentPresence, canonical_npc_place, settlement_business_operator,
    settlement_business_operator__view, settlement_resident_presence,
    settlement_resident_presence__view, settlement_resident_profile,
    settlement_resident_profile__view,
};

mod admission;
#[cfg(feature = "authority-tests")]
mod authority_tests;

type ResidentResult<T> = Result<T, String>;

#[derive(Clone, Debug)]
#[table(accessor = scene_resident_roster)]
pub struct SceneResidentRoster {
    #[primary_key]
    pub server: Identity,
    pub area: TacticalResidentArea,
    pub character_ids: Vec<u64>,
}

/// The trusted native compiler supplies actual playable businesses and venues,
/// never positions. The database resolves their existing Character identities.
#[derive(Clone, Debug, Eq, PartialEq, SpacetimeType)]
pub struct TacticalResidentArea {
    pub operator_character_ids: Vec<u64>,
    pub common_venues: Vec<SettlementVenueKind>,
}

/// Scheduling and identity facts for the serving process, not a second person
/// record. This carries no private personality, case truth or historical seed.
#[derive(Clone, Debug, SpacetimeType)]
pub struct TacticalResidentCandidate {
    pub character_id: u64,
    pub name: String,
    pub alive: bool,
    pub presence: SettlementResidentPresence,
    pub venue: Option<SettlementVenueKind>,
    pub business_id: Option<adventuresim_world_schema::settlement_buildings::BusinessId>,
}

impl TacticalResidentArea {
    fn includes_presence(
        &self,
        presence: &SettlementResidentPresence,
        business: Option<&adventuresim_world_schema::settlement_buildings::BusinessId>,
    ) -> bool {
        let Some(place) = canonical_npc_place(&presence.settlement_id, &presence.location_id)
        else {
            return false;
        };
        if let Some(business) = business {
            return Some(place)
                == canonical_npc_place(
                    &presence.settlement_id,
                    business_location(business.key.usage, business.key.ordinal),
                );
        }
        matches!(place, StrategicPlaceId::SettlementVenue { kind, .. }
            if self.common_venues.contains(&kind))
    }

    fn canonicalize(&mut self) -> ResidentResult<()> {
        if self.operator_character_ids.len() > MAX_LOCAL_SCENE_RESIDENTS
            || self.common_venues.len() > MAX_LOCAL_SCENE_RESIDENTS
        {
            return Err("Resident area exceeds its protocol work limit".into());
        }
        self.operator_character_ids.sort_unstable();
        self.common_venues.sort_unstable();
        if self.operator_character_ids.first() == Some(&0)
            || self
                .operator_character_ids
                .windows(2)
                .any(|ids| ids[0] == ids[1])
            || self
                .common_venues
                .windows(2)
                .any(|venues| venues[0] == venues[1])
        {
            return Err("Resident area contains invalid or repeated identities".into());
        }
        Ok(())
    }

    fn resolve(
        &self,
        ctx: &ReducerContext,
        settlement: &TacticalSettlementSnapshot,
    ) -> ResidentResult<Vec<u64>> {
        let mut selected = BTreeSet::new();
        let venues: BTreeSet<_> = self.common_venues.iter().copied().collect();
        for character_id in &self.operator_character_ids {
            if !settlement
                .operators
                .iter()
                .any(|operator| operator.operator_character_id == *character_id)
            {
                return Err(
                    "Selected operator is outside the captured settlement businesses".into(),
                );
            }
            if has_npc_authority(ctx, *character_id) {
                selected.insert(*character_id);
            }
        }
        let mut ambient = BTreeSet::new();
        for resident in ctx
            .db
            .settlement_resident_profile()
            .home_settlement_id()
            .filter(&settlement.id)
        {
            if !has_npc_authority(ctx, resident.character_id) {
                continue;
            }
            if ctx
                .db
                .settlement_business_operator()
                .operator_character_id()
                .find(resident.character_id)
                .is_some()
            {
                continue;
            }
            let Some(presence) = ctx
                .db
                .settlement_resident_presence()
                .character_id()
                .find(resident.character_id)
            else {
                continue;
            };
            let Some(StrategicPlaceId::SettlementVenue { kind, .. }) =
                canonical_npc_place(&presence.settlement_id, &presence.location_id)
            else {
                continue;
            };
            if venues.contains(&kind) {
                ambient.insert(resident.character_id);
            }
        }
        // Operators take precedence. Select a stable bounded subset of other
        // residents; a city's population must not prevent a small area opening.
        let remaining = MAX_LOCAL_SCENE_RESIDENTS.saturating_sub(selected.len());
        selected.extend(ambient.into_iter().take(remaining));
        Ok(selected.into_iter().collect())
    }
}

#[reducer]
pub fn configure_settlement_residents(
    ctx: &ReducerContext,
    mut area: TacticalResidentArea,
) -> ResidentResult<()> {
    let server = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
        .ok_or("Only a registered settlement host can select scene residents")?;
    let TacticalSession::Settlement(settlement) = &server.session else {
        return Err("Combat missions cannot configure an ordinary resident roster".into());
    };
    area.canonicalize()?;
    if let Some(roster) = ctx.db.scene_resident_roster().server().find(ctx.sender()) {
        return if roster.area == area {
            Ok(())
        } else {
            Err("The serving area's resident roster is immutable".into())
        };
    }
    let character_ids = area.resolve(ctx, settlement)?;
    ctx.db.scene_resident_roster().insert(SceneResidentRoster {
        server: ctx.sender(),
        area,
        character_ids,
    });
    Ok(())
}

#[view(accessor = scene_resident_candidates, public)]
pub fn scene_resident_candidates(ctx: &ViewContext) -> Vec<TacticalResidentCandidate> {
    let Some(server) = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
    else {
        return Vec::new();
    };
    let TacticalSession::Settlement(settlement) = server.session else {
        return Vec::new();
    };
    let Some(roster) = ctx.db.scene_resident_roster().server().find(ctx.sender()) else {
        return Vec::new();
    };
    roster
        .character_ids
        .into_iter()
        .filter_map(|character_id| {
            ctx.db.npc_policy().character_id().find(character_id)?;
            let resident = ctx
                .db
                .settlement_resident_profile()
                .character_id()
                .find(character_id)?;
            if resident.home_settlement_id != settlement.id {
                return None;
            }
            let character = ctx.db.character().id().find(character_id)?;
            let presence = ctx
                .db
                .settlement_resident_presence()
                .character_id()
                .find(character_id)?;
            if presence.settlement_id != settlement.id {
                return None;
            }
            let business_id = ctx
                .db
                .settlement_business_operator()
                .operator_character_id()
                .find(character_id)
                .map(|assignment| assignment.business_id);
            if !roster
                .area
                .includes_presence(&presence, business_id.as_ref())
                || !resident_business_matches(
                    &settlement,
                    &roster.area,
                    character_id,
                    business_id.as_ref(),
                )
            {
                return None;
            }
            Some(TacticalResidentCandidate {
                venue: match canonical_npc_place(&presence.settlement_id, &presence.location_id) {
                    Some(StrategicPlaceId::SettlementVenue { kind, .. }) => Some(kind),
                    _ => None,
                },
                character_id,
                name: character.name,
                alive: character.alive,
                presence,
                business_id,
            })
        })
        .collect()
}

pub(super) fn has_npc_authority(ctx: &ReducerContext, character_id: u64) -> bool {
    ctx.db
        .npc_policy()
        .character_id()
        .find(character_id)
        .is_some()
}

fn resident_business_matches(
    settlement: &TacticalSettlementSnapshot,
    area: &TacticalResidentArea,
    character_id: u64,
    business: Option<&adventuresim_world_schema::settlement_buildings::BusinessId>,
) -> bool {
    if !area.operator_character_ids.contains(&character_id) {
        return business.is_none();
    }
    settlement.operators.iter().any(|captured| {
        captured.operator_character_id == character_id && business == Some(&captured.business_id)
    })
}
