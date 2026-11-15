//! The game client: netahoy's prediction and the transports, the fighters' bodies and models,
//! input, the camera, impacts and the injury rings.
//!
//! This player's fighter is a [`PredictedPlayer`] stepped ahead on its own commands and drawn
//! by an [`OwnVisual`] that eases across corrections ([`smoothing`]). Every other fighter is a
//! [`RemoteVisual`] that netahoy places where its history had it at the remote render time; what
//! it was doing comes from its [`FighterAction`](crate::FighterAction) history at that time.

mod animation;
mod arena_view;
mod camera;
mod impacts;
mod input;
pub mod rig;
mod rings;
mod smoothing;

use bevy::prelude::*;
use bevy_netahoy::{
    History, NetAhoyClientPlugin, NetAhoyClientSystems, OwnPlayer, OwnVisual, PlayerId,
    PlayerShape, PlayerSnapshot, PredictedPlayer, RemotePlayer, RemoteVisual,
    transport::{NetAhoyTransportClientPlugin, ServerEndpoints},
};

use crate::{Fighter, fighter::fighter_body};

/// The game's client half. Add it after [`TacticalProtocolPlugin`](crate::TacticalProtocolPlugin),
/// with a window and rendering.
pub struct TacticalClientPlugin {
    /// Where the server is.
    pub endpoints: ServerEndpoints,
}

impl Plugin for TacticalClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NetAhoyClientPlugin::<Fighter>::default(),
            NetAhoyTransportClientPlugin::new(self.endpoints.clone()),
            input::InputPlugin,
            animation::AnimationPlugin,
            impacts::ImpactsPlugin,
            arena_view::ArenaViewPlugin,
            rings::RingsPlugin,
        ))
        .add_observer(spawn_prediction)
        .add_systems(
            Update,
            (
                spawn_remote_visuals,
                smoothing::place_own_visual,
                camera::follow,
            )
                .chain()
                .after(NetAhoyClientSystems::Interpolate),
        );
    }
}

/// This player's prediction and the visual that draws it, once the server says which fighter
/// is ours. netahoy gives the prediction its id and state; we give it a body.
fn spawn_prediction(
    ours: On<Add, OwnPlayer>,
    mut commands: Commands,
    shape: Res<PlayerShape>,
    predictions: Query<(), With<PredictedPlayer>>,
) {
    if !predictions.is_empty() {
        return;
    }
    let prediction = commands
        .spawn((
            Name::new("own fighter prediction"),
            PredictedPlayer {
                server_entity: ours.event_target(),
            },
            fighter_body(&shape),
        ))
        .id();
    commands.spawn((
        Name::new("own fighter"),
        OwnVisual {
            prediction_entity: prediction,
        },
        Transform::default(),
        Visibility::default(),
    ));
}

/// A visual for every other fighter, once it has a pose to stand at.
fn spawn_remote_visuals(
    mut commands: Commands,
    remotes: Query<(Entity, &PlayerId, &History<PlayerSnapshot>), With<RemotePlayer>>,
    visuals: Query<&RemoteVisual>,
) {
    for (server_entity, player_id, history) in &remotes {
        let Some((_, newest)) = history.newest() else {
            continue;
        };
        if visuals
            .iter()
            .any(|visual| visual.server_entity == server_entity)
        {
            continue;
        }
        commands.spawn((
            Name::new(format!("fighter {}", player_id.0)),
            RemoteVisual {
                server_entity,
                player_id: *player_id,
            },
            Transform::from_translation(newest.position),
            Visibility::default(),
        ));
    }
}
