//! Bots the server plays like players: every tick it queues each one a [`UserCmd`] before the
//! commands are stepped, so a bot fights through the same step, replication and lag
//! compensation as a client's fighter, and is drawn the same. A bot closes on the nearest
//! fighter, strikes when it is in reach, and raises its guard while the other one strikes,
//! though it misreads one strike in [`MISREAD_EVERY`]. No randomness: what it presses follows
//! from the tick and where its last step left everyone.

use bevy::prelude::*;
use bevy_netahoy::{
    Buttons, NetAhoyServerSystems, PlayerId, PlayerOwner, PlayerShape, PlayerState, QueuedUserCmds,
    ServerTick, SubtickFire, UserCmd,
};

use crate::{
    Fighter,
    input::GUARD,
    moves::strike::{REACH, STRIKE_TICKS, Strike},
    server::spawn_fighter,
};

/// How many bots the server runs unless told otherwise.
pub const BOT_COUNT: u64 = 2;
/// Bots' player ids start here, clear of the clients'.
const FIRST_BOT_ID: u64 = 9_001;
/// A bot strikes from a little inside its reach, so the target is still in it at contact.
const STRIKE_RANGE: f32 = REACH * 0.8;
/// Ticks a bot waits after a strike before the next, so it can be fought.
const REST_TICKS: u64 = STRIKE_TICKS as u64 * 2;
/// Each bot's first strike waits this many ticks more than the one before, so two bots facing
/// each other don't strike in lockstep.
const STAGGERED_START_TICKS: u64 = 7;
/// A bot misreads one strike in this many and doesn't guard it: two bots that guarded
/// everything would never land a blow on each other.
const MISREAD_EVERY: u32 = 3;

/// The bots, beside [`TacticalServerPlugin`](super::TacticalServerPlugin).
pub struct BotsPlugin {
    /// How many.
    pub count: u64,
}

impl Default for BotsPlugin {
    fn default() -> Self {
        Self { count: BOT_COUNT }
    }
}

/// How many bots to spawn.
#[derive(Resource, Clone, Copy)]
struct BotCount(u64);

impl Plugin for BotsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BotCount(self.count))
            .add_systems(Startup, spawn_bots)
            .add_systems(
                FixedPreUpdate,
                drive_bots.before(NetAhoyServerSystems::ApplyCommands),
            );
    }
}

/// A bot's own command count and when it may strike again.
#[derive(Component)]
struct Bot {
    sequence: u32,
    rest_until: u64,
}

fn spawn_bots(mut commands: Commands, shape: Res<PlayerShape>, count: Res<BotCount>) {
    for index in 0..count.0 {
        let id = PlayerId(FIRST_BOT_ID + index);
        let fighter = spawn_fighter(&mut commands, &shape, id);
        // Owned by nobody: a player with the command queue every player has, whose
        // OwnerSnapshot goes to no client.
        commands.entity(fighter).insert((
            PlayerOwner(Entity::PLACEHOLDER),
            Bot {
                sequence: 0,
                rest_until: index * STAGGERED_START_TICKS,
            },
        ));
    }
}

/// Each bot's command for this tick.
fn drive_bots(
    tick: Res<ServerTick>,
    mut bots: Query<(&PlayerId, &mut Bot, &mut QueuedUserCmds)>,
    fighters: Query<(&PlayerId, &PlayerState<Fighter>)>,
) {
    for (id, mut bot, mut queued) in &mut bots {
        let Some((_, me)) = fighters.iter().find(|(other, _)| *other == id) else {
            continue;
        };
        let target = fighters
            .iter()
            .filter(|(other, state)| *other != id && !state.game.down)
            .min_by(|(_, a), (_, b)| {
                let a = a.moves.position.distance_squared(me.moves.position);
                let b = b.moves.position.distance_squared(me.moves.position);
                a.total_cmp(&b)
            });
        let command = bot.think(tick.0, me, target.map(|(_, state)| state));
        queued.push(command);
    }
}

impl Bot {
    /// What the bot does this tick, at `me`, against `target`.
    fn think(
        &mut self,
        tick: u64,
        me: &PlayerState<Fighter>,
        target: Option<&PlayerState<Fighter>>,
    ) -> UserCmd {
        self.sequence = self.sequence.wrapping_add(1);
        let mut command = UserCmd {
            sequence: self.sequence,
            // It sees the world as it is now, as a client with no lag would.
            seen_server_tick: tick,
            ..default()
        };
        let Some(target) = target else {
            return command;
        };
        let offset = (target.moves.position - me.moves.position).with_y(0.0);
        let distance = offset.length();
        // Yaw 0 looks along -Z.
        let yaw = (-offset.x).atan2(-offset.z);
        command.look = Vec2::new(yaw, 0.0);
        if distance > STRIKE_RANGE {
            command.movement = Vec2::Y;
        }
        let mut buttons = Buttons::empty();
        let reads = |strike: Strike| !strike.thrown_on.is_multiple_of(MISREAD_EVERY);
        if target.game.strike.is_some_and(reads) && me.game.strike.is_none() {
            buttons |= GUARD;
        } else if distance <= STRIKE_RANGE && tick >= self.rest_until && me.game.can_act() {
            command.fire = Some(SubtickFire {
                frac: 0.0,
                look: command.look,
                origin: None,
            });
            self.rest_until = tick + REST_TICKS;
        }
        command.buttons = buttons;
        command
    }
}
