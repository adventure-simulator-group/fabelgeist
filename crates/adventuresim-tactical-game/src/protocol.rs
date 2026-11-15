//! What both peers register, in the same order: the netcode for [`Fighter`], the state others
//! see, and the server's verdicts.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_netahoy::{
    AppHistoryExt, KccSchedule, NetAhoyProtocolPlugin, PlayerShape, TickRate,
    bevy_ahoy::prelude::AhoyPlugins,
};
use bevy_replicon::prelude::*;

use crate::{FighterAction, StrikeOutcome, Vitals, arena, fighter::Fighter};

/// The simulation rate: one command, one step and one published pose per fighter per tick.
pub const TICK_RATE: TickRate = TickRate::new(20);

/// A fighter's body: 1.8 m tall, the fabelgeist rig's 1.73 m and a little headroom.
pub const FIGHTER_SHAPE: PlayerShape = PlayerShape {
    radius: 0.35,
    half_height: 0.9,
    eye_height: 0.7,
};

/// Physics, the character controller on netahoy's schedule, replicon and the tactical protocol.
/// Both binaries add it before their side's plugin.
pub struct TacticalProtocolPlugin;

impl Plugin for TacticalProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            // netahoy steps the controllers itself, on this schedule.
            AhoyPlugins::new(KccSchedule),
            RepliconPlugins,
            NetAhoyProtocolPlugin::<Fighter>::new(TICK_RATE).with_player_shape(FIGHTER_SHAPE),
        ))
        .replicate_with_history::<FighterAction>()
        .replicate::<Vitals>()
        .add_server_message::<StrikeOutcome>(Channel::Ordered)
        .add_systems(Startup, arena::spawn_colliders);
    }
}
