//! Keyboard and mouse into netahoy's [`ClientInput`]: WASD to move, the mouse to look while the
//! cursor is captured (click to capture, Esc to release), left click to strike, right click or
//! Shift held to guard, Space to jump.

use bevy::{
    input::{common_conditions::input_just_pressed, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_netahoy::{Buttons, ClientInput, NetAhoyClientSystems, SubtickFire};

use crate::input::GUARD;

/// Radians the view turns per pixel of mouse travel.
const MOUSE_SENSITIVITY: f32 = 0.0012;
/// How far the view tilts up or down (radians).
const PITCH_LIMIT: f32 = 1.2;

pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Look>()
            .add_systems(
                Update,
                (
                    capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
                    release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
                    turn_look,
                )
                    .chain(),
            )
            .add_systems(
                RunFixedMainLoop,
                gather_input.in_set(NetAhoyClientSystems::GatherInput),
            );
    }
}

/// Where the player looks: yaw and pitch, radians. The camera follows it too.
#[derive(Resource, Default, Clone, Copy)]
pub(super) struct Look {
    pub(super) yaw: f32,
    pub(super) pitch: f32,
}

impl Look {
    fn as_vec(self) -> Vec2 {
        Vec2::new(self.yaw, self.pitch)
    }
}

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
}

fn turn_look(
    cursor: Single<&CursorOptions>,
    motion: Res<AccumulatedMouseMotion>,
    mut look: ResMut<Look>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }
    look.yaw -= motion.delta.x * MOUSE_SENSITIVITY;
    look.pitch = (look.pitch - motion.delta.y * MOUSE_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
}

fn gather_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    cursor: Single<&CursorOptions>,
    look: Res<Look>,
    fixed_time: Res<Time<Fixed>>,
    mut input: ResMut<ClientInput>,
) {
    let key = |code| f32::from(u8::from(keys.pressed(code)));
    input.movement = Vec2::new(
        key(KeyCode::KeyD) - key(KeyCode::KeyA),
        key(KeyCode::KeyW) - key(KeyCode::KeyS),
    )
    .normalize_or_zero();
    input.look = look.as_vec();
    let mut buttons = Buttons::empty();
    buttons.set(Buttons::JUMP, keys.pressed(KeyCode::Space));
    buttons.set(
        GUARD,
        mouse.pressed(MouseButton::Right) || keys.pressed(KeyCode::ShiftLeft),
    );
    input.buttons = buttons;
    if keys.just_pressed(KeyCode::Space) {
        input.pressed |= Buttons::JUMP;
    }
    // The click that captures the cursor isn't a strike.
    let captured = cursor.grab_mode == CursorGrabMode::Locked;
    if captured && mouse.just_pressed(MouseButton::Left) {
        input.fire = Some(SubtickFire {
            frac: fixed_time.overstep_fraction(),
            look: look.as_vec(),
            origin: None,
        });
    }
}
