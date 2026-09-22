//! Character studio lighting and orbit navigation.
use adventuresim_character_creator::studio_environment;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::EguiContexts;

/// Right edge of the creator side panel in logical window pixels, so
/// pointer input over the menu never reaches the orbit camera.
#[derive(Resource, Default)]
pub(crate) struct CreatorPanelRight(pub(crate) f32);

/// Closest and farthest orbit distances, in metres.
const RADIUS: (f32, f32) = (0.25, 14.0);

#[derive(Component, Clone, Copy)]
pub(crate) struct OrbitCamera {
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) radius: f32,
    pub(crate) focus: Vec3,
    /// Where the camera glides to until the pointer takes over.
    pub(crate) goal: Option<OrbitGoal>,
}

/// A framing the orbit camera eases toward.
#[derive(Clone, Copy)]
pub(crate) struct OrbitGoal {
    pub(crate) focus: Vec3,
    pub(crate) radius: f32,
    /// Yaw and pitch to turn to; `None` keeps the current view direction.
    pub(crate) angles: Option<(f32, f32)>,
}

impl OrbitCamera {
    fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    /// Ease toward the goal; frame-rate independent.
    fn approach(&mut self, seconds: f32) {
        let Some(goal) = self.goal else { return };
        let t = 1.0 - (-seconds * 9.0).exp();
        self.focus = self.focus.lerp(goal.focus, t);
        self.radius += (goal.radius - self.radius) * t;
        if let Some((yaw, pitch)) = goal.angles {
            // Turn the short way round, however far the view was dragged.
            let turn = (yaw - self.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            self.yaw += turn * t;
            self.pitch += (pitch - self.pitch) * t;
        }
        if self.focus.distance(goal.focus) < 1e-3 && (self.radius - goal.radius).abs() < 1e-3 {
            self.goal = None;
        }
    }
}

pub(crate) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        Camera3d::default(),
        // The studio's soft boxes and room light everything indirectly, and
        // give metal something to reflect.
        GeneratedEnvironmentMapLight {
            environment_map: images.add(studio_environment::studio_environment()),
            intensity: studio_environment::STUDIO_ENVIRONMENT_INTENSITY,
            ..default()
        },
        Transform::default(),
        OrbitCamera {
            yaw: 0.1,
            pitch: -0.05,
            radius: 2.7,
            focus: Vec3::new(0.0, 1.0, 0.0),
            goal: None,
        },
    ));

    let light_position = Vec3::new(-2.4, 4.2, 3.0);
    commands.spawn((
        SpotLight {
            color: Color::srgb(1.0, 0.90, 0.79),
            intensity: 1_050_000.0,
            range: 8.0,
            radius: 0.38,
            inner_angle: 0.30,
            outer_angle: 0.58,
            shadow_maps_enabled: true,
            shadow_depth_bias: 0.025,
            shadow_normal_bias: 1.9,
            shadow_map_near_z: 0.1,
            ..default()
        },
        Transform::from_translation(light_position).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.17, 0.18),
            perceptual_roughness: 0.86,
            reflectance: 0.12,
            ..default()
        })),
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each pointer input, the UI state and the camera"
)]
pub(crate) fn orbit_camera(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    panel: Res<CreatorPanelRight>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let Ok((mut transform, mut orbit)) = camera.single_mut() else {
        return;
    };
    let pointer_over_panel = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .is_some_and(|cursor| cursor.x <= panel.0);
    let pointer_owned_by_ui = pointer_over_panel
        || contexts
            .ctx_mut()
            .is_ok_and(|context| context.egui_wants_pointer_input());
    let delta: Vec2 = motion.read().map(|event| event.delta).sum();
    let panning = buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle);
    if !pointer_owned_by_ui && delta != Vec2::ZERO {
        if buttons.pressed(MouseButton::Left) {
            orbit.goal = None;
            orbit.yaw -= delta.x * 0.007;
            orbit.pitch = (orbit.pitch - delta.y * 0.007).clamp(-1.2, 1.2);
        } else if panning {
            orbit.goal = None;
            let rotation = orbit.rotation();
            let scale = orbit.radius * 0.0011;
            orbit.focus += rotation * Vec3::new(-delta.x * scale, delta.y * scale, 0.0);
        }
    }
    if pointer_owned_by_ui {
        wheel.clear();
    } else {
        for event in wheel.read() {
            orbit.goal = None;
            orbit.radius = (orbit.radius * (-event.y * 0.1).exp()).clamp(RADIUS.0, RADIUS.1);
        }
    }
    orbit.approach(time.delta_secs());
    transform.translation = orbit.focus + orbit.rotation() * Vec3::new(0.0, 0.0, orbit.radius);
    transform.look_at(orbit.focus, Vec3::Y);
}

/// The orbit distance that shows `size` whole in the visible part of the
/// window, and the focus that centres `center` there rather than behind the panel.
pub(crate) fn framing(
    window: &Window,
    panel_right: f32,
    yaw: f32,
    center: Vec3,
    size: Vec3,
) -> (Vec3, f32) {
    let (width, height) = (window.width(), window.height().max(1.0));
    let visible = (width - panel_right).max(1.0);
    // Bevy's default vertical field of view.
    let tan = (std::f32::consts::FRAC_PI_4 / 2.0).tan();
    let radius = ((size.y * 0.5 / tan).max(size.x * 0.5 / (tan * visible / height)) * 1.12
        + size.z * 0.5)
        .clamp(RADIUS.0, RADIUS.1);
    let metres_per_pixel = 2.0 * radius * tan / height;
    let right = Quat::from_rotation_y(yaw) * Vec3::X;
    (
        center - right * panel_right * 0.5 * metres_per_pixel,
        radius,
    )
}
