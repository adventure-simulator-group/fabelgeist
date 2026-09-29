//! Character studio lighting and orbit navigation.
use adventuresim_character_creator::studio_environment;
use bevy::{
    camera::SubCameraView,
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

/// The generated character's rest-pose bounds, once there is one.
#[derive(Resource, Default)]
pub(crate) struct CharacterBounds(pub(crate) Option<(Vec3, Vec3)>);

impl CharacterBounds {
    /// The box around a generated body's vertices.
    pub(crate) fn of(positions: &[[f32; 3]]) -> Option<(Vec3, Vec3)> {
        positions
            .iter()
            .map(|p| Vec3::from_array(*p))
            .fold(None, |bounds, p| {
                Some(bounds.map_or((p, p), |(lo, hi): (Vec3, Vec3)| (lo.min(p), hi.max(p))))
            })
    }
}

/// A framing of the character that the creator's camera buttons ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shot {
    FullBody,
    Portrait,
    TurnLeft,
    TurnRight,
}

/// The shot to frame next, taken by [`frame_shot`].
#[derive(Resource, Default)]
pub(crate) struct ShotRequest(pub(crate) Option<Shot>);

/// Extra height framed around a shot, leaving room for the nameplate above
/// and the action bar below.
const SHOT_HEADROOM: f32 = 1.3;
/// Height framed by a portrait, from the crown down, in metres.
const PORTRAIT_HEIGHT: f32 = 0.42;
/// Width and depth framed by a portrait, in metres.
const PORTRAIT_BREADTH: f32 = 0.34;
/// A slight downward look that suits a standing figure.
const PRESENTATION_PITCH: f32 = -0.05;
/// How far one turn button rotates the view.
const TURN_STEP: f32 = std::f32::consts::FRAC_PI_4;

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
        DistanceFog {
            color: BACKDROP,
            falloff: FogFalloff::Linear {
                start: 7.0,
                end: 20.0,
            },
            ..default()
        },
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

    spawn_stage(&mut commands, &mut meshes, &mut materials);
}

/// Colour of the dark room around the stage, which the far floor fades into.
pub(crate) const BACKDROP: Color = Color::srgb(0.028, 0.027, 0.032);
/// Radius of the round dais the character stands on, in metres.
const DAIS_RADIUS: f32 = 0.72;
/// Height of the dais; its top is the ground the character stands on.
const DAIS_HEIGHT: f32 = 0.06;
/// Width of the bronze band around the dais, in metres.
const DAIS_BAND: f32 = 0.04;

/// A stone dais with a bronze band, on a floor that fades into the room.
fn spawn_stage(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let floor = -DAIS_HEIGHT;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(60.0, 60.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.075, 0.072, 0.07),
            perceptual_roughness: 0.92,
            reflectance: 0.08,
            ..default()
        })),
        Transform::from_xyz(0.0, floor, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(DAIS_RADIUS, DAIS_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.195, 0.19),
            perceptual_roughness: 0.8,
            reflectance: 0.15,
            ..default()
        })),
        Transform::from_xyz(0.0, floor + DAIS_HEIGHT * 0.5, 0.0),
    ));
    let band_height = DAIS_HEIGHT * 0.6;
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(DAIS_RADIUS + DAIS_BAND, band_height))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.62, 0.46, 0.24),
            metallic: 1.0,
            perceptual_roughness: 0.38,
            ..default()
        })),
        Transform::from_xyz(0.0, floor + band_height * 0.5, 0.0),
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

/// Glide the camera to the requested shot of the character, and to a full
/// body shot when the first character appears.
pub(crate) fn frame_shot(
    mut request: ResMut<ShotRequest>,
    mut introduced: Local<bool>,
    bounds: Res<CharacterBounds>,
    panel: Res<CreatorPanelRight>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<&mut OrbitCamera>,
) {
    // The panel's width is only known once it has been drawn.
    if panel.0 <= 0.0 {
        return;
    }
    if !*introduced && bounds.0.is_some() {
        request.0.get_or_insert(Shot::FullBody);
    }
    let (Some(shot), Some((lo, hi)), Ok(window), Ok(mut orbit)) =
        (request.0, bounds.0, windows.single(), camera.single_mut())
    else {
        return;
    };
    request.0 = None;
    *introduced = true;
    let front = Some((0.0, PRESENTATION_PITCH));
    orbit.goal = Some(match shot {
        Shot::FullBody => {
            let size = (hi - lo) * Vec3::new(1.0, SHOT_HEADROOM, 1.0);
            OrbitGoal {
                focus: (lo + hi) * 0.5,
                radius: framing_radius(window, panel.0, size),
                angles: front,
            }
        }
        Shot::Portrait => {
            let center = Vec3::new(
                (lo.x + hi.x) * 0.5,
                hi.y - PORTRAIT_HEIGHT * 0.5,
                (lo.z + hi.z) * 0.5,
            );
            let size = Vec3::new(
                PORTRAIT_BREADTH,
                PORTRAIT_HEIGHT * SHOT_HEADROOM,
                PORTRAIT_BREADTH,
            );
            OrbitGoal {
                focus: center,
                radius: framing_radius(window, panel.0, size),
                angles: front,
            }
        }
        Shot::TurnLeft | Shot::TurnRight => {
            // Continue from where a glide in progress is heading.
            let current = orbit.goal.unwrap_or(OrbitGoal {
                focus: orbit.focus,
                radius: orbit.radius,
                angles: None,
            });
            let (yaw, pitch) = current.angles.unwrap_or((orbit.yaw, orbit.pitch));
            let step = if shot == Shot::TurnLeft {
                -TURN_STEP
            } else {
                TURN_STEP
            };
            OrbitGoal {
                angles: Some((yaw + step, pitch)),
                ..current
            }
        }
    });
}

/// The orbit distance that shows `size` whole in the visible part of the
/// window. [`center_beside_panel`] keeps the focus centred there.
pub(crate) fn framing_radius(window: &Window, panel_right: f32, size: Vec3) -> f32 {
    let (width, height) = (window.width(), window.height().max(1.0));
    let visible = (width - panel_right).max(1.0);
    // Bevy's default vertical field of view.
    let tan = (std::f32::consts::FRAC_PI_4 / 2.0).tan();
    ((size.y * 0.5 / tan).max(size.x * 0.5 / (tan * visible / height)) * 1.12 + size.z * 0.5)
        .clamp(RADIUS.0, RADIUS.1)
}

/// Move the camera's optical centre from the window's middle to the middle of
/// the part beside the panel, so the orbit focus appears there and the camera
/// turns about it. The camera renders a view as much wider as the panel and
/// shows only its left part.
pub(crate) fn center_beside_panel(
    panel: Res<CreatorPanelRight>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<&mut Camera, With<OrbitCamera>>,
) {
    let (Ok(window), Ok(mut camera)) = (windows.single(), camera.single_mut()) else {
        return;
    };
    let size = UVec2::new(
        window.physical_width().max(1),
        window.physical_height().max(1),
    );
    let panel = (panel.0 * window.scale_factor()).round() as u32;
    let view = SubCameraView {
        full_size: UVec2::new(size.x + panel, size.y),
        offset: Vec2::ZERO,
        size,
    };
    if camera.sub_camera_view != Some(view) {
        camera.sub_camera_view = Some(view);
    }
}
