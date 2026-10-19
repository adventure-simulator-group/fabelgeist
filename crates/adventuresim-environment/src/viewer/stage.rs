//! The camera, the sun and the sky the forest is seen under.

use adventuresim_environment::render::ForestViewCamera;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

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

/// Where the sun stands, and whether it casts shadows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lighting {
    /// Around the vertical, radians.
    pub sun_azimuth: f32,
    /// Above the horizon, radians.
    pub sun_elevation: f32,
    pub shadows: bool,
}

impl Default for Lighting {
    /// The sun a scene starts with.
    fn default() -> Self {
        Self {
            sun_azimuth: 0.9,
            sun_elevation: 0.75,
            shadows: true,
        }
    }
}

/// The camera's view: the sky's haze and light. The live viewer orbits it
/// with the mouse; a snapshot places it once.
pub fn camera() -> impl Bundle {
    (
        Camera3d::default(),
        ForestViewCamera,
        distance_fog(),
        AmbientLight {
            color: SKY,
            brightness: SKY_BRIGHTNESS,
            ..default()
        },
    )
}

pub fn distance_fog() -> DistanceFog {
    DistanceFog {
        color: SKY,
        falloff: FogFalloff::from_visibility(HAZE_DISTANCE),
        ..default()
    }
}

/// The orbit a camera that opens before the first snapshot takes.
pub fn first_orbit() -> Orbit {
    Orbit::framing(FIRST_FRAMING)
}

pub fn sun(mut commands: Commands) {
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

/// Where an orbiting camera stands: around a focus, at a distance.
#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub focus: Vec3,
    /// Around the vertical, and above the horizon, radians.
    pub yaw: f32,
    pub pitch: f32,
    /// From the focus, metres.
    pub radius: f32,
}

impl Orbit {
    /// Frame the box from `bounds[0]` to `bounds[1]`:
    /// from above and to one side, with room for the plants to grow.
    pub fn framing(bounds: [[f32; 3]; 2]) -> Self {
        let [low, high] = bounds.map(Vec3::from_array);
        let width = (high.x - low.x).max(high.z - low.z);
        let high = Vec3::new(high.x, high.y.max(low.y + width * GROWTH_ROOM), high.z);
        let center = (low + high) * 0.5;
        Self {
            focus: Vec3::new(center.x, low.y + (high.y - low.y) * AIM, center.z),
            yaw: FRAMING_YAW,
            pitch: FRAMING_PITCH,
            radius: high.distance(low).max(4.0),
        }
    }

    /// The camera's transform, as `bevy_panorbit_camera` places it.
    pub fn transform(&self) -> Transform {
        let rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(-self.pitch);
        Transform {
            translation: self.focus + rotation * Vec3::new(0.0, 0.0, self.radius),
            rotation,
            ..default()
        }
    }

    /// Move `camera` here at once.
    pub fn apply(&self, camera: &mut PanOrbitCamera) {
        camera.focus = self.focus;
        camera.target_focus = self.focus;
        camera.radius = Some(self.radius);
        camera.target_radius = self.radius;
        camera.yaw = Some(self.yaw);
        camera.target_yaw = self.yaw;
        camera.pitch = Some(self.pitch);
        camera.target_pitch = self.pitch;
        camera.force_update = true;
    }
}
