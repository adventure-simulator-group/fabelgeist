//! Shared-time resident enrollment and acknowledged teardown while players visit.

use std::time::Duration;

use adventuresim_core::{strategic_presence::DailyPresenceWindow, strategic_time::official_minute};
use adventuresim_stdb_client::{TacticalResidentArea, TacticalResidentCandidate};
use adventuresim_tactical_core::prelude::{CharacterId, ItemOf, TacticalCombatState};
use adventuresim_world_schema::calendar::StrategicMinute;
use bevy::{ecs::system::SystemParam, prelude::*};
use spacetimedb_sdk::Timestamp;

use super::actors::{ResidentClaim, ResidentConfiguration, ResidentPhase};
use super::reachability::ResidentPlacementWorld;
use super::{CommonResidentStations, OperatorStations, ResidentActor, ResidentActors};
use crate::{
    local_time::LocalVisits,
    settlement_session::RetainedPlayerFilter,
    stdb::{SceneAdmissions, SpacetimeDb},
};

const RESIDENT_ADMISSION_INTERVAL: Duration = Duration::from_secs(1);
const RESIDENT_ADMISSION_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(SystemParam)]
pub(super) struct ResidentLifecycle<'w, 's> {
    commands: Commands<'w, 's>,
    conn: Res<'w, SpacetimeDb>,
    visits: ResMut<'w, LocalVisits>,
    actors: Query<'w, 's, &'static TacticalCombatState, With<ResidentActor>>,
    items: Query<'w, 's, (Entity, &'static ItemOf)>,
    entities: Query<'w, 's, ()>,
    time: Res<'w, Time<Real>>,
    exit: MessageWriter<'w, AppExit>,
}

pub(super) fn reconcile(
    mut lifecycle: ResidentLifecycle,
    stations: Option<Res<OperatorStations>>,
    common: Option<Res<CommonResidentStations>>,
    mut residents: ResMut<ResidentActors>,
    mut admissions: ResMut<SceneAdmissions>,
    players: Query<(), RetainedPlayerFilter>,
    placement: ResidentPlacementWorld,
) {
    let (Some(stations), Some(common)) = (stations, common) else {
        return;
    };
    if let Err(error) = lifecycle.reconcile(
        &stations,
        &common,
        &mut residents,
        &admissions,
        !players.is_empty(),
        &placement,
    ) {
        admissions.close();
        error!(%error, "Resident lifecycle lost scene authority; releasing serving connection");
        lifecycle.exit.write(AppExit::error());
    }
}

impl ResidentLifecycle<'_, '_> {
    fn reconcile(
        &mut self,
        stations: &OperatorStations,
        common: &CommonResidentStations,
        residents: &mut ResidentActors,
        admissions: &SceneAdmissions,
        occupied: bool,
        placement: &ResidentPlacementWorld,
    ) -> Result {
        if !self.configuration_is_ready(residents, stations, common)? {
            return Ok(());
        }
        let Some(epoch) = self.conn.world_clock_epoch() else {
            return Ok(());
        };
        let minute = official_minute(epoch, Timestamp::now().to_micros_since_unix_epoch());
        let candidates: Vec<_> = self
            .conn
            .resident_candidates()
            .into_iter()
            .filter(|candidate| available(candidate, minute))
            .collect();
        let occupied = occupied && !admissions.is_closing();
        self.depart(residents, &candidates, occupied)?;
        if !occupied
            || self
                .time
                .elapsed()
                .saturating_sub(residents.last_admission_check)
                < RESIDENT_ADMISSION_INTERVAL
        {
            return Ok(());
        }
        residents.last_admission_check = self.time.elapsed();
        for candidate in candidates {
            let character = CharacterId(candidate.character_id);
            if residents.contains(character) || admissions.actors.contains_key(&character) {
                continue;
            }
            let position = match placement.choose(&candidate, stations, common, residents) {
                Ok(position) => position,
                Err(reason) => {
                    if residents.site_failures.insert(character, reason) != Some(reason) {
                        info!(?character, ?reason, "Resident station is unavailable");
                    }
                    continue;
                }
            };
            residents.site_failures.remove(&character);
            self.conn.request_resident(character)?;
            let entity = self.commands.spawn_empty().id();
            residents.claims.insert(
                character,
                ResidentClaim {
                    entity,
                    position,
                    phase: ResidentPhase::Admitting(self.time.elapsed()),
                },
            );
        }
        Ok(())
    }

    fn configuration_is_ready(
        &self,
        residents: &mut ResidentActors,
        stations: &OperatorStations,
        common: &CommonResidentStations,
    ) -> Result<bool, BevyError> {
        match residents.configuration {
            ResidentConfiguration::Unconfigured => {
                self.conn.configure_residents(TacticalResidentArea {
                    operator_character_ids: stations
                        .0
                        .iter()
                        .map(|station| station.character.0)
                        .collect(),
                    common_venues: common.venues(),
                })?;
                residents.configuration = ResidentConfiguration::Pending(self.time.elapsed());
                Ok(false)
            }
            ResidentConfiguration::Pending(started) => {
                if self.time.elapsed().saturating_sub(started) >= RESIDENT_ADMISSION_TIMEOUT {
                    return Err("Resident area configuration acknowledgement timed out".into());
                }
                Ok(false)
            }
            ResidentConfiguration::Ready => Ok(true),
        }
    }

    fn depart(
        &mut self,
        residents: &mut ResidentActors,
        candidates: &[TacticalResidentCandidate],
        occupied: bool,
    ) -> Result {
        let mut released = Vec::new();
        for (character, claim) in &mut residents.claims {
            if matches!(claim.phase, ResidentPhase::Removing) {
                if !self.entities.contains(claim.entity) {
                    released.push(*character);
                }
                continue;
            }
            if let ResidentPhase::Admitting(started) = claim.phase {
                if self.time.elapsed().saturating_sub(started) >= RESIDENT_ADMISSION_TIMEOUT {
                    return Err(
                        format!("Resident {} admission profile timed out", character.0).into(),
                    );
                }
                continue;
            }
            if !occupied
                || !candidates
                    .iter()
                    .any(|candidate| candidate.character_id == character.0)
            {
                claim.phase = ResidentPhase::Departing;
            }
            if !matches!(claim.phase, ResidentPhase::Departing) {
                continue;
            }
            let fatigue = self
                .actors
                .get(claim.entity)
                .ok()
                .map(|state| state.fatigue);
            if !self
                .visits
                .depart(*character, fatigue, self.time.elapsed(), &self.conn)?
            {
                continue;
            }
            self.visits.release(*character);
            for (item, owner) in &self.items {
                if owner.0 == claim.entity {
                    self.commands.entity(item).despawn();
                }
            }
            self.commands.entity(claim.entity).despawn();
            claim.phase = ResidentPhase::Removing;
        }
        for character in released {
            residents.claims.remove(&character);
        }
        Ok(())
    }
}

fn available(candidate: &TacticalResidentCandidate, minute: StrategicMinute) -> bool {
    candidate.alive
        && DailyPresenceWindow {
            start_minute: candidate.presence.start_minute,
            end_minute: candidate.presence.end_minute,
        }
        .remaining_minutes(
            minute,
            candidate.presence.context_suppressed,
            candidate.presence.health_suppressed,
        )
        .is_ok()
}
