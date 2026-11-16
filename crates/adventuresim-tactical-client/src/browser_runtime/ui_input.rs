//! Suppress physical controls while an HTML document owns interaction.
//!
//! This runs before enhanced input reads the devices, and also clears the raw
//! input used by indirect orders. It does not pause any simulation systems.

use adventuresim_tactical_core::prelude::bevy_enhanced_input::EnhancedInputSystems;
use bevy::{
    input::{
        InputSystems,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    },
    prelude::*,
};
use wasm_bindgen::prelude::*;

pub(super) struct BrowserUiInputPlugin;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = tacticalUiOwnsInput)]
    fn interface_owns_input() -> bool;
}

impl Plugin for BrowserUiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            suppress_interface_input
                .after(InputSystems)
                .before(EnhancedInputSystems::Prepare),
        );
    }
}

fn suppress_interface_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
) {
    if interface_owns_input() {
        keys.reset_all();
        mouse.reset_all();
        *motion = default();
        *scroll = default();
    }
}
