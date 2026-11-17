//! Wall-calendar projection and acknowledged ordinary-scene departures.

use std::{collections::HashMap, time::Duration};

use adventuresim_core::{
    provisioning::LocalNeeds,
    strategic_time::{CalendarMinuteFraction, PersonalClock, official_minute},
    tactical_session::LocalSceneVisitId,
};
use adventuresim_stdb_client::{ConnectedPlayer, LocalSceneVisitExit, TacticalActorTime};
use adventuresim_tactical_core::prelude::{CharacterId, SceneEnvironment, TacticalCombatState};
use adventuresim_world_schema::calendar::StrategicMinute;
use bevy::prelude::*;
use spacetimedb_sdk::Timestamp;

use crate::{combat::CombatSet, player_projection::PlayerProjectionSet, stdb::SpacetimeDb};

const DEPARTURE_RETRY_INTERVAL: Duration = Duration::from_secs(1);
const DEPARTURE_ACK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Resource, Default)]
pub(crate) struct LocalVisits {
    actors: HashMap<CharacterId, LiveVisit>,
}

struct LiveVisit {
    id: LocalSceneVisitId,
    entered_at: Timestamp,
    starting: PersonalClock,
    needs: LocalNeeds,
    departure: Departure,
}

struct PendingDeparture {
    exit: LocalSceneVisitExit,
    started: Duration,
    last_attempt: Duration,
}

enum Departure {
    Live,
    Pending(PendingDeparture),
    Accepted,
}

impl LocalVisits {
    pub(crate) fn capture(&mut self, player: &ConnectedPlayer) -> Result {
        let TacticalActorTime::LocalVisit(clock) = &player.time else {
            return Ok(());
        };
        let character = CharacterId(player.character.id);
        let id = LocalSceneVisitId::try_from(clock.visit_id)?;
        if let Some(existing) = self.actors.get(&character) {
            if existing.id != id {
                return Err("Actor profile changed its live time owner".into());
            }
            return Ok(());
        }
        self.actors.insert(
            character,
            LiveVisit {
                id,
                entered_at: clock.entered_at,
                starting: PersonalClock::new(
                    StrategicMinute::new(clock.personal_clock.minute.minutes),
                    CalendarMinuteFraction::try_from(clock.personal_clock.fraction.numerator)?,
                ),
                needs: LocalNeeds::from_balances_kcal_ml(
                    clock.food_balance_kcal,
                    clock.water_balance_ml,
                )?,
                departure: Departure::Live,
            },
        );
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.actors.is_empty()
    }

    /// Freeze the final body once and retain it through ambiguous acknowledgements.
    /// `now` is Bevy's real clock, independent of debug simulation-speed controls.
    pub(crate) fn depart(
        &mut self,
        character: CharacterId,
        fatigue: Option<f32>,
        now: Duration,
        conn: &SpacetimeDb,
    ) -> Result<bool, String> {
        let visit = self
            .actors
            .get_mut(&character)
            .ok_or("Departing actor has no live visit")?;
        if matches!(visit.departure, Departure::Accepted) {
            return Ok(true);
        }
        if matches!(visit.departure, Departure::Live) {
            let exit = match fatigue {
                Some(value) => {
                    let checked =
                        adventuresim_core::essential::FatigueIncapacitation::try_from(value)
                            .map_err(|error| error.to_string())?;
                    LocalSceneVisitExit::Projected(checked.fraction())
                }
                None => LocalSceneVisitExit::Unprojected,
            };
            visit.departure = Departure::Pending(PendingDeparture {
                exit,
                started: now,
                last_attempt: now.saturating_sub(DEPARTURE_RETRY_INTERVAL),
            });
        }
        let Departure::Pending(pending) = &mut visit.departure else {
            return Ok(false);
        };
        if now.saturating_sub(pending.started) >= DEPARTURE_ACK_TIMEOUT {
            return Err(
                "Local departure acknowledgement timed out; release the server connection".into(),
            );
        }
        if now.saturating_sub(pending.last_attempt) >= DEPARTURE_RETRY_INTERVAL {
            pending.last_attempt = now;
            if let Err(error) = conn.finish_local_visit(character, visit.id, pending.exit.clone()) {
                warn!(%error, "Could not queue local departure; retaining the same receipt");
            }
        }
        Ok(false)
    }

    pub(crate) fn acknowledge(
        &mut self,
        character: CharacterId,
        id: LocalSceneVisitId,
        result: Result<(), String>,
    ) {
        let Some(visit) = self.actors.get_mut(&character) else {
            return;
        };
        if visit.id != id || !matches!(visit.departure, Departure::Pending(_)) {
            return;
        }
        match result {
            Ok(()) => visit.departure = Departure::Accepted,
            Err(error) => {
                warn!(%error, "Local departure not acknowledged; retaining the same receipt")
            }
        }
    }

    pub(crate) fn release(&mut self, character: CharacterId) {
        self.actors.remove(&character);
    }

    fn project(
        &self,
        character: CharacterId,
        now: Timestamp,
        state: &mut TacticalCombatState,
    ) -> Result {
        let Some(visit) = self.actors.get(&character) else {
            return Ok(());
        };
        if !matches!(visit.departure, Departure::Live) {
            return Ok(());
        }
        // A serving host can briefly observe an entry timestamp in its future
        // because its UTC clock differs from the database host. Preview no
        // elapsed time yet; the database timestamp remains closure authority.
        let elapsed = now.duration_since(visit.entered_at).unwrap_or_default();
        let ending = visit.starting.advance_real_time(elapsed)?;
        let needs = visit.needs.advance_between(visit.starting, ending)?;
        // These sources already belong to the enrollment total. Replace their
        // contribution so the next condition derivation counts them exactly once.
        state.starting_incapacitation =
            (state.starting_incapacitation - state.starting_hunger - state.starting_thirst
                + needs.hunger()
                + needs.thirst())
            .max(0.0);
        state.starting_hunger = needs.hunger();
        state.starting_thirst = needs.thirst();
        Ok(())
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<LocalVisits>().add_systems(
        Update,
        project_live_time
            .after(PlayerProjectionSet::Spawn)
            .before(CombatSet::Condition)
            .run_if(is_local_scene),
    );
}

fn is_local_scene(args: Res<crate::Args>) -> bool {
    matches!(
        args.instance_id,
        adventuresim_core::tactical_session::TacticalInstanceId::Settlement(_)
    )
}

fn project_live_time(
    visits: Res<LocalVisits>,
    conn: Res<SpacetimeDb>,
    mut actors: Query<(&CharacterId, &mut TacticalCombatState)>,
    mut retained: Query<(
        &crate::player_projection::DisconnectedPlayer,
        &mut crate::player_projection::DisconnectedProjection,
    )>,
    environments: Query<(Entity, &SceneEnvironment)>,
    mut commands: Commands,
) -> Result {
    let now = Timestamp::now();
    for (character, mut state) in &mut actors {
        visits.project(*character, now, &mut state)?;
    }
    for (player, mut projection) in &mut retained {
        if let Some(state) = projection.combat_state_mut() {
            visits.project(player.character_id, now, state)?;
        }
    }
    if let Some(epoch) = conn.world_clock_epoch() {
        let minute = official_minute(epoch, now.to_micros_since_unix_epoch());
        for (entity, environment) in &environments {
            if environment.absolute_minute != minute {
                let mut next = environment.clone();
                next.absolute_minute = minute;
                next.lunar_phase_minute = minute;
                commands.entity(entity).insert(next);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_world_schema::calendar::{MINUTES_PER_DAY, WORLD_START_MINUTE};

    fn retained_visit(id: u64) -> LiveVisit {
        LiveVisit {
            id: LocalSceneVisitId::try_from(id).unwrap(),
            entered_at: Timestamp::from_micros_since_unix_epoch(0),
            starting: PersonalClock::new(WORLD_START_MINUTE, CalendarMinuteFraction::ZERO),
            needs: LocalNeeds::from_balances_kcal_ml(0.0, 0.0).unwrap(),
            departure: Departure::Live,
        }
    }

    #[test]
    fn delayed_departure_acknowledgement_cannot_close_a_later_visit() {
        let character = CharacterId(1);
        let old_id = LocalSceneVisitId::try_from(1).unwrap();
        let mut visits = LocalVisits::default();
        visits.actors.insert(character, retained_visit(2));
        visits.acknowledge(character, old_id, Ok(()));
        assert!(matches!(
            visits.actors[&character].departure,
            Departure::Live
        ));
        let pending = PendingDeparture {
            exit: LocalSceneVisitExit::Projected(0.25),
            started: Duration::ZERO,
            last_attempt: Duration::ZERO,
        };
        visits.actors.get_mut(&character).unwrap().departure = Departure::Pending(pending);
        let current_id = LocalSceneVisitId::try_from(2).unwrap();
        visits.acknowledge(character, current_id, Err("ambiguous callback".into()));
        let Departure::Pending(still_pending) = &visits.actors[&character].departure else {
            panic!("Lost the retry receipt")
        };
        assert_eq!(still_pending.exit, LocalSceneVisitExit::Projected(0.25));
        visits.acknowledge(character, current_id, Ok(()));
        assert!(matches!(
            visits.actors[&character].departure,
            Departure::Accepted
        ));
    }

    #[test]
    fn panels_and_repeated_frames_replace_need_sources_without_double_counting() {
        let character = CharacterId(1);
        let mut visits = LocalVisits::default();
        visits.actors.insert(character, retained_visit(1));
        let elapsed_micros =
            adventuresim_core::strategic_time::real_micros_for_official_minutes(MINUTES_PER_DAY);
        let now = Timestamp::from_micros_since_unix_epoch(i64::try_from(elapsed_micros).unwrap());
        let mut state = TacticalCombatState {
            starting_incapacitation: 0.4,
            ..default()
        };
        visits.project(character, now, &mut state).unwrap();
        let once = state.clone();
        for _ in 0..1_000 {
            visits.project(character, now, &mut state).unwrap();
        }
        // Rounding budget for replacing two f32 source contributions near one.
        let source_rounding = 0.000_001;
        assert!(
            (state.starting_incapacitation - once.starting_incapacitation).abs() < source_rounding
        );
        assert_eq!(state.starting_hunger, once.starting_hunger);
        assert_eq!(state.starting_thirst, once.starting_thirst);
        assert_eq!(state.starting_thirst, 1.0);
        assert!(state.starting_incapacitation > 1.0);
        assert_eq!(state.fatigue, once.fatigue);
    }
}
