//! Purpose-specific resources. Ordinary scenes do not own mission objectives.

use std::time::Duration;

use adventuresim_core::tactical_session::TacticalInstanceId;
use bevy::prelude::*;

use crate::{
    cli::Args,
    mission::MissionState,
    player_projection::{DisconnectedPlayer, ReconnectSession},
    stdb::{SceneAdmissions, SceneReady, SpacetimeDb},
};

/// Time to connect after provisioning an otherwise empty area. Reconnects use
/// the existing retained-player grace before an occupied area becomes empty.
const INITIAL_ENTRY_GRACE: Duration = Duration::from_secs(120);
const RETIREMENT_ACK_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) type RetainedPlayerFilter = Or<(With<ReconnectSession>, With<DisconnectedPlayer>)>;

#[derive(Resource)]
pub(crate) struct SettlementSession {
    lifecycle: SettlementLifecycle,
}

enum SettlementLifecycle {
    AwaitingEntry(Timer),
    Occupied,
    Retiring(Timer),
}

pub(crate) fn configure(app: &mut App, args: &Args) -> Result<(), String> {
    match &args.instance_id {
        TacticalInstanceId::Mission(_) => {
            let expected = args
                .expected_party_members
                .ok_or("Mission scenes require --expected-party-members")?;
            let kills = args
                .required_enemy_kills
                .ok_or("Mission scenes require --required-enemy-kills")?;
            let timeout = if args.no_timeout {
                None
            } else {
                let duration = Duration::try_from_secs_f32(args.timeout)
                    .map_err(|error| format!("Invalid mission timeout: {error}"))?;
                Some(Timer::new(duration, TimerMode::Once))
            };
            let reconnect_grace = Duration::try_from_secs_f32(args.party_reconnect_grace)
                .map_err(|error| format!("Invalid reconnect grace: {error}"))?;
            app.insert_resource(MissionState::new(timeout, kills, expected, reconnect_grace));
        }
        TacticalInstanceId::Settlement(_) => {
            if args.expected_party_members.is_some()
                || args.required_enemy_kills.is_some()
                || args.enemy_fixture.is_some()
            {
                return Err("Settlement scenes cannot carry mission objectives or enemies".into());
            }
            app.insert_resource(SettlementSession {
                lifecycle: SettlementLifecycle::AwaitingEntry(Timer::new(
                    INITIAL_ENTRY_GRACE,
                    TimerMode::Once,
                )),
            });
            app.add_systems(
                Update,
                retire_empty_scene.run_if(resource_exists::<SceneReady>),
            );
        }
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects scene lifetime, retained actors and durable visit closure resources separately"
)]
fn retire_empty_scene(
    mut session: ResMut<SettlementSession>,
    time: Res<Time<Real>>,
    players: Query<(), RetainedPlayerFilter>,
    mut admissions: ResMut<SceneAdmissions>,
    conn: Res<SpacetimeDb>,
    visits: Res<crate::local_time::LocalVisits>,
    residents: Res<crate::residents::ResidentActors>,
    mut exit: MessageWriter<AppExit>,
) -> Result {
    if let SettlementLifecycle::Retiring(wait) = &mut session.lifecycle {
        if wait.tick(time.delta()).is_finished() {
            error!("Settlement retirement acknowledgement timed out; releasing connection");
            exit.write(AppExit::error());
        }
        return Ok(());
    }
    if !players.is_empty() || !visits.is_empty() || !residents.is_empty() {
        session.lifecycle = SettlementLifecycle::Occupied;
        return Ok(());
    }
    if let SettlementLifecycle::AwaitingEntry(wait) = &mut session.lifecycle
        && !wait.tick(time.delta()).is_finished()
    {
        return Ok(());
    }
    conn.retire_settlement()?;
    admissions.close();
    session.lifecycle =
        SettlementLifecycle::Retiring(Timer::new(RETIREMENT_ACK_TIMEOUT, TimerMode::Once));
    Ok(())
}
