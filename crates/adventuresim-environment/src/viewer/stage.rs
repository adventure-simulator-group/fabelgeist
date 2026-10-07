//! The camera, the sun and the sky the forest is seen under.

use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::scene_controls::Lighting;

/// The clear sky behind the scene, sRGB.
pub const SKY: Color = Color::srgb(0.62, 0.74, 0.88);
/// The sky's light from every direction, as Bevy's ambient brightness at its
/// default exposure: enough to read the shade, not to flatten the sun.
const SKY_BRIGHTNESS: f32 = 500.0;
/// How far the sun's shadows reach from the camera, metres: a whole scene.
const SHADOW_REACH: f32 = 600.0;
/// Haze over distance: how far the air lets one see, metres.
const HAZE_DISTANCE: f32 = 4_000.0;
/// A camera that opens before the first snapshot frames the scene looks at
/// this box, metres.
const FIRST_FRAMING: [[f32; 3]; 2] = [[-30.0, 0.0, -30.0], [30.0, 20.0, 30.0]];
/// A framed scene is seen from above and to one side, radians.
const FRAMING_YAW: f32 = 0.7;
const FRAMING_PITCH: f32 = 0.42;
/// Room for what will grow: a young scene is framed as if its plants were
/// already this share of its width tall.
const GROWTH_ROOM: f32 = 0.35;
/// The camera aims at this share of the framed box's height.
const AIM: f32 = 0.3;

/// The sun.
#[derive(Component)]
pub struct Sun;

pub fn setup(mut commands: Commands) {
    let mut camera = PanOrbitCamera::default();
    frame(&mut camera, FIRST_FRAMING);
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        camera,
        DistanceFog {
            color: SKY,
            falloff: FogFalloff::from_visibility(HAZE_DISTANCE),
            ..default()
        },
        AmbientLight {
            color: SKY,
            brightness: SKY_BRIGHTNESS,
            ..default()
        },
    ));
    commands.spawn((
        Sun,
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            maximum_distance: SHADOW_REACH,
            ..default()
        }
        .build(),
        sun_transform(Lighting::default()),
    ));
}

/// The sun's transform for `lighting`: shining from its azimuth and
/// elevation.
pub fn sun_transform(lighting: Lighting) -> Transform {
    let (sin_azimuth, cos_azimuth) = lighting.sun_azimuth.sin_cos();
    let (sin_elevation, cos_elevation) = lighting.sun_elevation.sin_cos();
    let towards_sun = Vec3::new(
        cos_elevation * sin_azimuth,
        sin_elevation,
        cos_elevation * cos_azimuth,
    );
    Transform::default().looking_to(-towards_sun, Vec3::Y)
}

/// Frame the box from `bounds[0]` to `bounds[1]`:
/// from above and to one side, with room for the plants to grow.
pub fn frame(camera: &mut PanOrbitCamera, bounds: [[f32; 3]; 2]) {
    let [low, high] = bounds.map(Vec3::from_array);
    let width = (high.x - low.x).max(high.z - low.z);
    let high = Vec3::new(high.x, high.y.max(low.y + width * GROWTH_ROOM), high.z);
    let center = (low + high) * 0.5;
    let extent = high.distance(low).max(4.0);
    aim(
        camera,
        Vec3::new(center.x, low.y + (high.y - low.y) * AIM, center.z),
        extent,
    );
    camera.target_yaw = FRAMING_YAW;
    camera.target_pitch = FRAMING_PITCH;
    camera.yaw = Some(FRAMING_YAW);
    camera.pitch = Some(FRAMING_PITCH);
}

/// Look at `focus` from `distance` away, keeping the angle.
pub fn aim(camera: &mut PanOrbitCamera, focus: Vec3, distance: f32) {
    camera.focus = focus;
    camera.target_focus = focus;
    camera.radius = Some(distance);
    camera.target_radius = distance;
    camera.force_update = true;
}
