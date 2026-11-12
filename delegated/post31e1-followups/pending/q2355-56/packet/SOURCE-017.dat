//! Orbit, proportional zoom and camera-relative panning for every exhibit.

use crate::presentation::TacticalGameplayCamera;
use bevy::prelude::*;
use bevy::{camera::Exposure, light::AmbientLight};

pub(super) const VERTICAL_FOV_DEGREES: f32 = 45.0;
const ORBIT_RADIANS_PER_PIXEL: f32 = 0.006;
const ZOOM_LOG_SCALE_PER_PIXEL: f32 = 0.001;
const MAX_PITCH_RADIANS: f32 = 1.35;
const STUDIO_EXPOSURE_EV100: f32 = 12.5;
const STUDIO_AMBIENT_BRIGHTNESS: f32 = 155.0;
const PAN_DISTANCE_PER_SECOND: f32 = 0.6;
const MAX_INPUT_SECONDS: f32 = 0.05;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CameraSpace {
    Studio,
    Landscape,
}

#[derive(Clone, Resource)]
pub(super) struct OrbitView {
    pub focus: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub limits: std::ops::RangeInclusive<f32>,
    pub space: CameraSpace,
}

impl Default for OrbitView {
    fn default() -> Self {
        Self::new(Vec3::ZERO, 1.0, CameraSpace::Studio)
    }
}

impl OrbitView {
    pub fn new(focus: Vec3, distance: f32, space: CameraSpace) -> Self {
        Self {
            focus,
            distance,
            yaw: 0.45,
            pitch: 0.16,
            limits: distance * 0.45..=distance * 2.5,
            space,
        }
    }

    pub fn orbit(&mut self, x: f32, y: f32) {
        self.yaw = (self.yaw - x * ORBIT_RADIANS_PER_PIXEL).rem_euclid(std::f32::consts::TAU);
        let minimum = if self.space == CameraSpace::Landscape {
            0.05
        } else {
            -MAX_PITCH_RADIANS
        };
        self.pitch = (self.pitch + y * ORBIT_RADIANS_PER_PIXEL).clamp(minimum, MAX_PITCH_RADIANS);
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance * (delta * ZOOM_LOG_SCALE_PER_PIXEL).exp())
            .clamp(*self.limits.start(), *self.limits.end());
    }

    pub fn pan(&mut self, right: f32, forward: f32, seconds: f32) {
        let input = Vec2::new(right, forward).clamp_length_max(1.0);
        let right = Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());
        let forward = if self.space == CameraSpace::Landscape {
            Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos())
        } else {
            Vec3::new(
                -self.yaw.sin() * self.pitch.sin(),
                self.pitch.cos(),
                -self.yaw.cos() * self.pitch.sin(),
            )
        };
        self.focus += (right * input.x + forward * input.y)
            * self.distance
            * PAN_DISTANCE_PER_SECOND
            * seconds.clamp(0.0, MAX_INPUT_SECONDS);
    }
}

pub(super) fn apply(
    mut commands: Commands,
    view: Res<OrbitView>,
    mut cameras: Query<(Entity, &mut Transform), With<TacticalGameplayCamera>>,
) {
    for (entity, mut camera) in &mut cameras {
        if view.space == CameraSpace::Studio {
            if view.is_changed() {
                commands.entity(entity).insert(AmbientLight {
                    color: Color::srgb(0.78, 0.84, 0.94),
                    brightness: STUDIO_AMBIENT_BRIGHTNESS,
                    ..default()
                });
            }
        } else if view.is_changed() {
            commands.entity(entity).remove::<AmbientLight>();
        }
        let direction = Vec3::new(
            view.yaw.sin() * view.pitch.cos(),
            view.pitch.sin(),
            view.yaw.cos() * view.pitch.cos(),
        );
        *camera = Transform::from_translation(view.focus + direction * view.distance)
            .looking_at(view.focus, Vec3::Y);
    }
}

// Production interior adaptation runs after transform propagation. Apply the
// studio exposure afterward, leaving outdoor adaptation in charge of landscapes.
pub(super) fn studio_exposure(
    view: Res<OrbitView>,
    mut cameras: Query<&mut Exposure, With<TacticalGameplayCamera>>,
) {
    if view.space == CameraSpace::Studio {
        for mut exposure in &mut cameras {
            exposure.ev100 = STUDIO_EXPOSURE_EV100;
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub(super) fn native_input(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<bevy::input::mouse::AccumulatedMouseMotion>,
    scroll: Res<bevy::input::mouse::AccumulatedMouseScroll>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut view: ResMut<OrbitView>,
) {
    if buttons.pressed(MouseButton::Left) {
        view.orbit(motion.delta.x, motion.delta.y);
    }
    const NATIVE_WHEEL_PIXELS_PER_LINE: f32 = -100.0;
    view.zoom(scroll.delta.y * NATIVE_WHEEL_PIXELS_PER_LINE);
    let axis = |positive, negative| {
        i32::from(keys.pressed(positive)) as f32 - i32::from(keys.pressed(negative)) as f32
    };
    view.pan(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        axis(KeyCode::KeyW, KeyCode::KeyS),
        time.delta_secs(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pan_follows_camera_and_has_no_diagonal_speed_bonus() {
        let mut view = OrbitView::new(Vec3::ZERO, 10.0, CameraSpace::Landscape);
        view.yaw = std::f32::consts::FRAC_PI_2;
        view.pan(0.0, 1.0, 0.05);
        assert!(view.focus.x < 0.0);
        assert_eq!(view.focus.y, 0.0);
        let straight = view.focus.length();
        view.focus = Vec3::ZERO;
        view.pan(1.0, 1.0, 0.05);
        assert!((view.focus.length() - straight).abs() < 0.0001);
        view.space = CameraSpace::Studio;
        view.focus = Vec3::ZERO;
        view.pan(0.0, 1.0, 0.05);
        assert!(view.focus.y > 0.0);
    }
    #[test]
    fn extreme_input_preserves_a_finite_camera_and_zoom_recovers() {
        let mut view = OrbitView::default();
        view.zoom(f32::MAX);
        assert_eq!(view.distance, *view.limits.end());
        view.zoom(-100.0);
        assert!(view.distance < *view.limits.end());
        view.orbit(500.0, f32::MAX);
        assert!(view.pitch < std::f32::consts::FRAC_PI_2);
        view.zoom(-f32::MAX);
        assert!(view.distance > 0.0);
    }
}
