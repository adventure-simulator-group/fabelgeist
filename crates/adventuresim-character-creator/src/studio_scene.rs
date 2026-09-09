//! Character studio lighting and orbit navigation.
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use bevy_egui::EguiContexts;

#[derive(Component)]
pub(crate) struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    radius: f32,
    focus: Vec3,
}

pub(crate) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        // A restrained ambient term stands in for indirect room bounce. It
        // prevents fully black occlusion without flattening the spotlight's
        // form and floor shadow.
        AmbientLight {
            color: Color::srgb(0.78, 0.84, 0.94),
            brightness: 155.0,
            ..default()
        },
        Transform::default(),
        OrbitCamera {
            yaw: 0.1,
            pitch: -0.05,
            radius: 2.7,
            focus: Vec3::new(0.0, 1.0, 0.0),
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

pub(crate) fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let Ok((mut transform, mut orbit)) = camera.single_mut() else {
        return;
    };
    let pointer_owned_by_ui = contexts
        .ctx_mut()
        .is_ok_and(|context| context.egui_wants_pointer_input());
    if buttons.pressed(MouseButton::Left) && !pointer_owned_by_ui {
        for event in motion.read() {
            orbit.yaw -= event.delta.x * 0.007;
            orbit.pitch = (orbit.pitch - event.delta.y * 0.007).clamp(-1.2, 1.2);
        }
    } else {
        motion.clear();
    }
    if pointer_owned_by_ui {
        wheel.clear();
    } else {
        for event in wheel.read() {
            orbit.radius = (orbit.radius * (-event.y * 0.1).exp()).clamp(1.2, 6.0);
        }
    }
    let rotation = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0);
    transform.translation = orbit.focus + rotation * Vec3::new(0.0, 0.0, orbit.radius);
    transform.look_at(orbit.focus, Vec3::Y);
}
