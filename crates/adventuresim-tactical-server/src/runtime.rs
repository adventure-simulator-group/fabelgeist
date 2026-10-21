//! The tactical authority is headless even when shared dependencies enable rendering.

use bevy::prelude::*;

pub(crate) struct ServerRuntimePlugin;

impl Plugin for ServerRuntimePlugin {
    fn build(&self, app: &mut App) {
        // Cargo feature unification must never select server presentation plugins.
        // MinimalPlugins keeps the existing unthrottled schedule runner and task pool.
        app.add_plugins((
            MinimalPlugins,
            bevy::app::PanicHandlerPlugin,
            bevy::log::LogPlugin {
                filter: "adventuresim_tactical_server=info,quickstep_trace=info,bevy_app=warn,bevy_ecs=warn".to_owned(),
                ..default()
            },
            bevy::transform::TransformPlugin,
            bevy::diagnostic::DiagnosticsPlugin,
            bevy::input::InputPlugin,
            bevy::app::TerminalCtrlCHandlerPlugin,
        ));
        #[cfg(feature = "debug")]
        app.add_plugins((
            bevy::asset::AssetPlugin::default(),
            bevy::world_serialization::WorldSerializationPlugin,
            bevy::scene::ScenePlugin,
        ));
    }
}
