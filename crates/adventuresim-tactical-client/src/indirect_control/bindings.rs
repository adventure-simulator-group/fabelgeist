//! Runtime settings for the single-character indirect-control scheme.

use bevy::prelude::*;

/// Bindings may be inspected and changed through Bevy's reflected resources.
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource, Default)]
pub(crate) struct IndirectBindings {
    pub toggle: KeyCode,
    pub stop: KeyCode,
    pub hold: KeyCode,
    pub cancel: KeyCode,
    pub order_modifier: KeyCode,
    pub waypoint_modifiers: [KeyCode; 2],
    pub orbit_modifiers: [KeyCode; 2],
    pub selection_modifiers: [KeyCode; 2],
    pub select: MouseButton,
    pub order: MouseButton,
    pub orbit: MouseButton,
    pub pan_forward: KeyCode,
    pub pan_backward: KeyCode,
    pub pan_left: KeyCode,
    pub pan_right: KeyCode,
}

impl Default for IndirectBindings {
    fn default() -> Self {
        Self {
            toggle: KeyCode::F8,
            stop: KeyCode::KeyX,
            hold: KeyCode::KeyH,
            cancel: KeyCode::Escape,
            order_modifier: KeyCode::Space,
            waypoint_modifiers: [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            orbit_modifiers: [KeyCode::AltLeft, KeyCode::AltRight],
            selection_modifiers: [KeyCode::ControlLeft, KeyCode::ControlRight],
            select: MouseButton::Left,
            order: MouseButton::Right,
            orbit: MouseButton::Middle,
            pan_forward: KeyCode::KeyW,
            pan_backward: KeyCode::KeyS,
            pan_left: KeyCode::KeyA,
            pan_right: KeyCode::KeyD,
        }
    }
}

pub(super) fn key_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Space => "Space",
        KeyCode::Escape => "Esc",
        KeyCode::F8 => "F8",
        KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift",
        KeyCode::AltLeft | KeyCode::AltRight => "Alt",
        KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl",
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        _ => "assigned key",
    }
}

pub(super) fn mouse_label(button: MouseButton) -> &'static str {
    match button {
        MouseButton::Left => "left click",
        MouseButton::Right => "right click",
        MouseButton::Middle => "middle click",
        _ => "assigned mouse button",
    }
}
