//! The game server: a fighter for each client that joins, the server's verdict on every strike
//! the steps report, getting fighters up again, and bots to fight. Headless, one fixed tick per
//! frame, so every tick's snapshot goes out.

mod bots;

use aeronet::io::connection::Disconnected;
use bevy::prelude::*;
use bevy_netahoy::{
    JoinRequest, NetAhoyServerPlugin, NetAhoyServerSystems, PlayerId, PlayerOwner, PlayerShape,
    PlayerState, ServerTick, TickRate,
};
use bevy_replicon::prelude::*;

pub use bots::{BOT_COUNT, BotsPlugin};

use crate::{
    CombatEvent, Fighter, FighterAction, Injury, StrikeOutcome, Vitals, arena,
    combat::{Contact, StrikeResult},
    fighter::{STAGGER_TICKS, fighter_body},
    moves::strike::StrikeKind,
};

/// Seconds a downed fighter stays down before getting up at a spawn point, unhurt.
pub const DOWN_SECONDS: f64 = 3.0;
/// How fast a hit knocks its victim away (m/s, across the ground).
const KNOCK_SPEED: f32 = 3.5;
/// How fast a guarded strike shoves (m/s).
const SHOVE_SPEED: f32 = 1.5;

/// The injury each strike deals.
pub fn strike_injury(kind: StrikeKind) -> Injury {
    match kind {
        StrikeKind::Swing => Injury::from_basis_points(2_000),
        StrikeKind::Thrust => Injury::from_basis_points(1_500),
    }
}

/// The game's server rules: netahoy's server, joins and leaves, strike outcomes, getting up,
/// the actions others see. A server that plays over the network adds netahoy's
/// `NetAhoyTransportServerPlugin` beside it; a test runs it bare. Add [`BotsPlugin`] for bots.
pub struct TacticalServerPlugin;

impl Plugin for TacticalServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NetAhoyServerPlugin::<Fighter>::default())
            .add_observer(join)
            .add_observer(leave)
            .add_systems(
                FixedPreUpdate,
                resolve_strikes.in_set(NetAhoyServerSystems::Events),
            )
            .add_systems(FixedPostUpdate, (get_up, record_actions).chain());
    }
}

/// The components a fighter needs on the server, besides its owner.
pub fn spawn_fighter(commands: &mut Commands, shape: &PlayerShape, id: PlayerId) -> Entity {
    commands
        .spawn((
            Name::new(format!("fighter {}", id.0)),
            id,
            PlayerState::<Fighter>::at(arena::spawn_point(id.0)),
            FighterAction::default(),
            Vitals::default(),
            fighter_body(shape),
        ))
        .id()
}

/// A fighter for each client that joins, once.
fn join(
    join: On<FromClient<JoinRequest>>,
    mut commands: Commands,
    shape: Res<PlayerShape>,
    players: Query<(&PlayerOwner, &PlayerId)>,
) {
    let Some(client) = join.client_id.entity() else {
        return;
    };
    if players.iter().any(|(owner, _)| owner.0 == client) {
        return;
    }
    let taken: Vec<u64> = players.iter().map(|(_, id)| id.0).collect();
    let id = PlayerId(
        (1..)
            .find(|id| !taken.contains(id))
            .expect("player ids run out past u64::MAX"),
    );
    info!("client {client} joined as fighter {}", id.0);
    let fighter = spawn_fighter(&mut commands, &shape, id);
    commands.entity(fighter).insert(PlayerOwner(client));
}

/// A client's fighter leaves with it.
fn leave(gone: On<Disconnected>, mut commands: Commands, players: Query<(Entity, &PlayerOwner)>) {
    for (player, owner) in &players {
        if owner.0 == gone.event_target() {
            commands.entity(player).despawn();
        }
    }
}

/// When a downed fighter gets up.
#[derive(Component, Clone, Copy, Debug)]
pub struct Downed {
    /// The tick they get up on.
    pub until: u64,
}

/// Applies every strike the steps reported to the one it reached: a guarded one shoves, any
/// other hurts, staggers and knocks back, and puts them down at [`Injury::DOWN`]. Every client
/// hears the verdict.
pub fn resolve_strikes(
    mut events: MessageReader<CombatEvent>,
    mut fighters: Query<(Entity, &PlayerId, &mut PlayerState<Fighter>, &mut Vitals)>,
    mut outcomes: MessageWriter<ToClients<StrikeOutcome>>,
    tick: Res<ServerTick>,
    rate: Res<TickRate>,
    mut commands: Commands,
) {
    for event in events.read() {
        let CombatEvent::Struck(contact) = *event else {
            continue;
        };
        let Some((entity, _, mut state, mut vitals)) = fighters
            .iter_mut()
            .find(|(_, id, _, _)| **id == contact.victim)
        else {
            continue;
        };
        if state.game.down {
            continue;
        }
        let result = apply_contact(&contact, &mut state, &mut vitals);
        info!(
            "strike {}#{} on fighter {}: {result:?}",
            contact.id.owner.0, contact.id.sequence, contact.victim.0
        );
        if result == StrikeResult::Downed {
            commands.entity(entity).insert(Downed {
                until: tick.0 + rate.ticks(DOWN_SECONDS),
            });
        }
        outcomes.write(ToClients {
            targets: SendTargets::CLIENTS_ONLY,
            message: StrikeOutcome {
                strike: contact.id,
                victim: contact.victim,
                point: contact.point,
                result,
            },
        });
    }
}

/// What `contact` does to the fighter it reached.
pub fn apply_contact(
    contact: &Contact,
    victim: &mut PlayerState<Fighter>,
    vitals: &mut Vitals,
) -> StrikeResult {
    let away = (victim.moves.position - contact.from)
        .with_y(0.0)
        .try_normalize()
        .unwrap_or(Vec3::X);
    if contact.guarded {
        victim.moves.velocity += away * SHOVE_SPEED;
        return StrikeResult::Guarded;
    }
    vitals.injury = vitals.injury.worsened_by(strike_injury(contact.kind));
    victim.moves.velocity += away * KNOCK_SPEED;
    victim.game.strike = None;
    victim.game.guarding = false;
    victim.game.stagger = STAGGER_TICKS;
    if vitals.injury.is_down() {
        victim.game.down = true;
        StrikeResult::Downed
    } else {
        StrikeResult::Hurt(vitals.injury)
    }
}

/// Gets every downed fighter whose time is up back on their feet at their spawn point, unhurt.
fn get_up(
    mut commands: Commands,
    tick: Res<ServerTick>,
    mut downed: Query<(
        Entity,
        &PlayerId,
        &Downed,
        &mut PlayerState<Fighter>,
        &mut Vitals,
    )>,
) {
    for (entity, id, downed, mut state, mut vitals) in &mut downed {
        if tick.0 < downed.until {
            continue;
        }
        *state = PlayerState::at(arena::spawn_point(id.0));
        *vitals = Vitals::default();
        commands.entity(entity).remove::<Downed>();
    }
}

/// What each fighter's tick left, for its history: in the fixed schedule, before netahoy
/// records it.
fn record_actions(mut fighters: Query<(&PlayerState<Fighter>, &mut FighterAction)>) {
    for (state, mut action) in &mut fighters {
        action.set_if_neq(FighterAction(state.game));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_netahoy::EventId;

    fn contact(guarded: bool) -> Contact {
        Contact {
            id: EventId {
                owner: PlayerId(1),
                sequence: 7,
            },
            kind: StrikeKind::Swing,
            victim: PlayerId(2),
            point: Vec3::new(0.0, 1.3, -1.0),
            from: Vec3::ZERO,
            guarded,
        }
    }

    fn victim() -> PlayerState<Fighter> {
        PlayerState::at(Vec3::new(0.0, 0.0, -1.0))
    }

    #[test]
    fn a_hit_hurts_staggers_and_knocks_away() {
        let mut state = victim();
        let mut vitals = Vitals::default();
        let result = apply_contact(&contact(false), &mut state, &mut vitals);
        assert_eq!(result, StrikeResult::Hurt(strike_injury(StrikeKind::Swing)));
        assert_eq!(state.game.stagger, STAGGER_TICKS);
        assert!(
            state.moves.velocity.z < 0.0,
            "knocked away from the attacker"
        );
    }

    #[test]
    fn a_guarded_strike_only_shoves() {
        let mut state = victim();
        let mut vitals = Vitals::default();
        let result = apply_contact(&contact(true), &mut state, &mut vitals);
        assert_eq!(result, StrikeResult::Guarded);
        assert_eq!(vitals.injury, Injury::NONE);
        assert_eq!(state.game.stagger, 0);
    }

    #[test]
    fn enough_hits_put_a_fighter_down() {
        let mut state = victim();
        let mut vitals = Vitals::default();
        let hits = Injury::DOWN.fraction() / strike_injury(StrikeKind::Swing).fraction();
        let mut result = StrikeResult::Guarded;
        for _ in 0..hits.ceil() as usize {
            result = apply_contact(&contact(false), &mut state, &mut vitals);
        }
        assert_eq!(result, StrikeResult::Downed);
        assert!(state.game.down);
    }
}
