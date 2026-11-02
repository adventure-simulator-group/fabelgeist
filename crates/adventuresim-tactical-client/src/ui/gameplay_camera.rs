//! Binds tactical UI to the camera the player actually sees.
use bevy::prelude::*;
use bevy_egui::{EguiContext, PrimaryEguiContext};

/// Pins egui and `bevy_ui` to the gameplay camera. Egui's first-camera adoption
/// is disabled in `UiPlugin::build`, so cameras that render offscreen never
/// receive UI passes. Left alone, `bevy_ui` picks the highest-order window
/// camera even when it is inactive, as the browser's forge preview camera is
/// during tactical play.
pub(super) fn pin_ui_to_gameplay_camera(
    mut commands: Commands,
    cameras: Query<
        Entity,
        (
            Added<crate::presentation::TacticalGameplayCamera>,
            Without<EguiContext>,
        ),
    >,
) {
    for camera in &cameras {
        commands
            .entity(camera)
            .insert((PrimaryEguiContext, IsDefaultUiCamera));
    }
}
