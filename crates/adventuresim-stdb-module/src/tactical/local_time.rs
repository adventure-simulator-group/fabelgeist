//! One time owner per ordinary admission, with an immutable closure receipt.

use adventuresim_core::{
    essential::FatigueIncapacitation, strategic_time::PersonalClock,
    tactical_session::LocalSceneVisitId,
};
use spacetimedb::Timestamp;

use super::*;
use crate::{condition::character_needs, time::character_time};

mod effects;

#[cfg(feature = "authority-tests")]
mod authority_tests;

type VisitResult<Value> = Result<Value, String>;

/// Operational interval metadata; never a copy of the tactical actor's HP,
/// position, damage, equipment actions or simulation ticks.
#[derive(Clone, Debug)]
#[table(accessor = local_scene_visit)]
pub struct LocalSceneVisit {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub character_id: u64,
    #[index(btree)]
    pub server: Identity,
    pub owner: LocalSceneTimeOwner,
    pub entered_at: Timestamp,
    pub starting_clock: PersonalClock,
    pub food_balance_kcal: f32,
    pub water_balance_ml: f32,
    pub state: LocalSceneVisitState,
}

#[derive(Clone, Debug, SpacetimeType)]
pub struct LocalSceneVisitClock {
    pub visit_id: u64,
    pub entered_at: Timestamp,
    pub personal_clock: PersonalClock,
    pub food_balance_kcal: f32,
    pub water_balance_ml: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, SpacetimeType)]
pub struct LocalSceneVisitClosure {
    pub closed_at: Timestamp,
    pub personal_clock: PersonalClock,
    pub receipt: LocalSceneVisitReceipt,
}

pub(super) struct ActorSceneProjection {
    pub(super) role: TacticalActorRole,
    pub(super) time: TacticalActorTime,
}

/// Participation remains fixed for a visit even if NPC authority transfers to
/// a player. The old resident visit must close before a player can claim it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, SpacetimeType)]
pub enum LocalSceneTimeOwner {
    Player,
    Resident,
}

#[derive(Clone, Debug, SpacetimeType)]
pub enum TacticalActorTime {
    MissionSnapshot,
    LocalVisit(LocalSceneVisitClock),
}

#[derive(Clone, Debug, PartialEq, Eq, SpacetimeType)]
pub enum LocalSceneVisitState {
    Open,
    Closed(LocalSceneVisitClosure),
}

#[derive(Clone, Debug, PartialEq, Eq, SpacetimeType)]
pub enum LocalSceneVisitReceipt {
    Requested(Vec<u8>),
    Interrupted,
}

/// Only the final general-fatigue consequence crosses this handoff. Live HP
/// and wounds remain transient; their medical handoff is owned separately.
#[derive(Clone, Debug, PartialEq, SpacetimeType)]
pub enum LocalSceneVisitExit {
    Unprojected,
    Projected { fatigue: f32 },
}

#[derive(Clone, Copy)]
enum DepartureFatigue {
    Unprojected,
    Projected(FatigueIncapacitation),
}

impl LocalSceneVisit {
    pub(super) fn begin(
        ctx: &ReducerContext,
        character_id: u64,
        server: Identity,
        owner: LocalSceneTimeOwner,
    ) -> VisitResult<Self> {
        if let Some(visit) = Self::active(ctx, character_id)? {
            if visit.server != server {
                return Err("Character time belongs to another live scene".into());
            }
            if visit.owner != owner {
                return Err(
                    "Character is already admitted with another scene participation role".into(),
                );
            }
            return Ok(visit);
        }
        crate::time::initialize_character_time(ctx, character_id)?;
        crate::condition::initialize_character_condition(ctx, character_id)?;
        let clock = ctx
            .db
            .character_time()
            .character_id()
            .find(character_id)
            .ok_or("Character time is missing")?;
        let needs = ctx
            .db
            .character_needs()
            .character_id()
            .find(character_id)
            .ok_or("Character needs are missing")?;
        if !needs.food_balance_kcal.is_finite() || !needs.water_balance_ml.is_finite() {
            return Err("Character needs are not finite".into());
        }
        let visit = ctx.db.local_scene_visit().insert(Self {
            id: 0, // SpacetimeDB assigns the positive storage key.
            character_id,
            server,
            owner,
            entered_at: ctx.timestamp,
            starting_clock: PersonalClock::new(clock.minutes, clock.fraction),
            food_balance_kcal: needs.food_balance_kcal,
            water_balance_ml: needs.water_balance_ml,
            state: LocalSceneVisitState::Open,
        });
        LocalSceneVisitId::try_from(visit.id).map_err(|error| error.to_string())?;
        Ok(visit)
    }

    fn active(ctx: &ReducerContext, character_id: u64) -> VisitResult<Option<Self>> {
        let mut visits = ctx
            .db
            .local_scene_visit()
            .character_id()
            .filter(character_id)
            .filter(|visit| matches!(visit.state, LocalSceneVisitState::Open));
        let visit = visits.next();
        if visits.next().is_some() {
            return Err("Character has multiple live time owners".into());
        }
        Ok(visit)
    }

    fn complete(
        mut self,
        ctx: &ReducerContext,
        receipt: LocalSceneVisitReceipt,
        fatigue: DepartureFatigue,
    ) -> VisitResult<()> {
        if let LocalSceneVisitState::Closed(closed) = &self.state {
            return if closed.receipt == receipt {
                Ok(())
            } else {
                Err("Scene visit was already closed with another receipt".into())
            };
        }
        let character = ctx
            .db
            .character()
            .id()
            .find(self.character_id)
            .ok_or("Scene visit character is missing")?;
        if character.server != self.server {
            return Err("Scene visit no longer owns this character".into());
        }
        let elapsed = ctx
            .timestamp
            .duration_since(self.entered_at)
            .ok_or("Scene visit wall clock moved backwards")?;
        let clock = self
            .starting_clock
            .advance_real_time(elapsed)
            .map_err(|error| error.to_string())?;
        effects::apply(ctx, &self, clock, fatigue)?;
        self.state = LocalSceneVisitState::Closed(LocalSceneVisitClosure {
            closed_at: ctx.timestamp,
            personal_clock: clock,
            receipt,
        });
        let server = self.server;
        let character_id = self.character_id;
        ctx.db.local_scene_visit().id().update(self);
        // Chronology can settle death or continuity. Never overwrite those
        // committed changes with the character snapshot from before effects.
        let character = ctx
            .db
            .character()
            .id()
            .find(character_id)
            .ok_or("Scene visit character disappeared during closure")?;
        if !character.has_tactical_server_assignment() {
            return Ok(());
        }
        super::entry::leave_tactical_scene_for_server(ctx, character, server)
    }
}

#[reducer]
pub fn finish_local_scene_visit(
    ctx: &ReducerContext,
    visit_id: u64,
    exit: LocalSceneVisitExit,
) -> VisitResult<()> {
    let id = LocalSceneVisitId::try_from(visit_id).map_err(|error| error.to_string())?;
    let visit = ctx
        .db
        .local_scene_visit()
        .id()
        .find(id.get())
        .ok_or("Scene visit is missing")?;
    if ctx.sender() != visit.server {
        return Err("Only the owning tactical server can close a scene visit".into());
    }
    let fatigue = match exit {
        LocalSceneVisitExit::Unprojected => DepartureFatigue::Unprojected,
        LocalSceneVisitExit::Projected { fatigue } => DepartureFatigue::Projected(
            FatigueIncapacitation::try_from(fatigue).map_err(|error| error.to_string())?,
        ),
    };
    let bytes = spacetimedb::sats::bsatn::to_vec(&exit)
        .map_err(|error| format!("Invalid scene exit receipt: {error}"))?;
    let receipt = LocalSceneVisitReceipt::Requested(Sha256::digest(bytes).to_vec());
    if matches!(visit.state, LocalSceneVisitState::Open) {
        let server = ctx
            .db
            .tactical_server_authority()
            .identity()
            .find(ctx.sender())
            .ok_or("Scene visit server is not registered")?;
        if !matches!(server.session, TacticalSession::Settlement(_)) {
            return Err("Local visits require an ordinary settlement server".into());
        }
    }
    visit.complete(ctx, receipt, fatigue)
}

pub(crate) fn require_unleased_time(ctx: &ReducerContext, character_id: u64) -> VisitResult<()> {
    if owns_character_time(ctx, character_id)? {
        return Err("Character time is owned by a live settlement scene".into());
    }
    Ok(())
}

pub(crate) fn owns_character_time(ctx: &ReducerContext, character_id: u64) -> VisitResult<bool> {
    Ok(LocalSceneVisit::active(ctx, character_id)?.is_some())
}

pub(super) fn interrupt_visit(
    ctx: &ReducerContext,
    character_id: u64,
    server: Identity,
) -> VisitResult<()> {
    let visit = LocalSceneVisit::active(ctx, character_id)?
        .ok_or("Assigned settlement character has no open visit")?;
    if visit.server != server {
        return Err("Interrupted visit belongs to another server".into());
    }
    visit.complete(
        ctx,
        LocalSceneVisitReceipt::Interrupted,
        DepartureFatigue::Unprojected,
    )
}

pub(super) fn actor_projection(
    ctx: &ViewContext,
    character_id: u64,
    server: &TacticalServer,
) -> Option<ActorSceneProjection> {
    if let TacticalSession::Mission(mission) = &server.session {
        let role = if mission.enemy_character_ids.contains(&character_id) {
            TacticalActorRole::Enemy
        } else {
            TacticalActorRole::Player
        };
        return Some(ActorSceneProjection {
            role,
            time: TacticalActorTime::MissionSnapshot,
        });
    }
    let mut visits = ctx
        .db
        .local_scene_visit()
        .character_id()
        .filter(character_id)
        .filter(|visit| matches!(visit.state, LocalSceneVisitState::Open));
    let visit = visits.next()?;
    if visits.next().is_some() || visit.server != server.identity {
        return None;
    }
    let role = match visit.owner {
        LocalSceneTimeOwner::Player => TacticalActorRole::Player,
        LocalSceneTimeOwner::Resident => TacticalActorRole::Resident,
    };
    Some(ActorSceneProjection {
        role,
        time: TacticalActorTime::LocalVisit(LocalSceneVisitClock {
            visit_id: visit.id,
            entered_at: visit.entered_at,
            personal_clock: visit.starting_clock,
            food_balance_kcal: visit.food_balance_kcal,
            water_balance_ml: visit.water_balance_ml,
        }),
    })
}
