//! The game half of a player: what a fighter keeps besides its movement, and the step that runs
//! one command through it.

use avian3d::prelude::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_netahoy::{
    History, PLAYER_COLLISION_LAYER, PlayerGame, PlayerId, PlayerShape, PlayerState, Recorded,
    Step, WORLD_COLLISION_LAYER,
    bevy_ahoy::{CharacterController, moves::Wish},
};
use serde::{Deserialize, Serialize};

use crate::{
    CombatEvent,
    moves::{guard, strike::Strike},
};

/// Ticks a hurt fighter reels for.
pub const STAGGER_TICKS: u16 = 8;

/// A fighter's state besides its movement. `Copy`, no heap: prediction and rollback copy it
/// whole every tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, MapEntities)]
pub struct Fighter {
    /// The strike under way, if any.
    pub strike: Option<Strike>,
    /// Fists up ([`guard`]).
    pub guarding: bool,
    /// Ticks left reeling from a hit: no moving, striking or guarding. The server sets it.
    pub stagger: u16,
    /// Down: no acting until the server gets them up.
    pub down: bool,
}

impl Fighter {
    /// Whether the fighter can act on its command this tick.
    pub fn can_act(&self) -> bool {
        self.stagger == 0 && !self.down
    }
}

/// What a step reads besides its fighter: every fighter's [`FighterAction`] history, sampled at
/// the command's seen time (a guard is met where the attacker saw it).
pub type FighterContext =
    Query<'static, 'static, (&'static PlayerId, &'static History<FighterAction>)>;

impl PlayerGame for Fighter {
    type Event = CombatEvent;
    type Context = FighterContext;

    fn step(player: &mut PlayerState<Self>, step: &mut Step<Self>) {
        let PlayerState { moves, game } = player;
        if game.can_act() {
            step.pmove(moves);
        } else {
            // Reeling or down: the body still falls and slides, the stick does nothing.
            step.with_pm(moves.position, |movers, pm| {
                bevy_netahoy::pmove::pmove(moves, Wish::default(), movers, pm);
            });
        }
        step.separate(moves);
        Strike::step(game, moves, step);
        guard::step(game, step.command());
        game.stagger = game.stagger.saturating_sub(1);
    }
}

/// A fighter as others see it, recorded every tick on the server and kept in a [`History`] on
/// both peers: remote fighters are drawn from it at the remote render time, and a strike meets
/// a guard where its attacker saw it.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FighterAction(pub Fighter);

impl Recorded for FighterAction {}

/// The controller and collision every fighter has, on both peers: the world only, since
/// `Step::separate` pushes fighters apart where each saw the other.
pub fn fighter_body(shape: &PlayerShape) -> impl Bundle {
    (
        CharacterController {
            filter: SpatialQueryFilter::from_mask(WORLD_COLLISION_LAYER),
            speed: 5.0,
            acceleration_hz: 10.0,
            friction_hz: 6.0,
            gravity: 23.0,
            ..default()
        },
        shape.collider(),
        CollisionLayers::new(PLAYER_COLLISION_LAYER, LayerMask::ALL),
    )
}
