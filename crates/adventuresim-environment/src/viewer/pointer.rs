//! The world under the pointer: where the tool in hand would act, marked on
//! the ground, and what a click there does. A drag still turns the camera.

use adventuresim_environment::render::GroundPart;
use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings};
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use fabelgeist_botany::viewer::sim::Command;

use crate::icons;
use crate::live::Viewer;
use crate::theme;
use crate::tools::{self, Tool};

/// A press that moves the pointer less than this is a click, not an orbit,
/// logical pixels.
const CLICK_SLACK: f32 = 5.0;
/// The sowing marker's radius, metres.
const SEED_RING: f32 = 1.5;
/// The sowing marker's stem, metres.
const SEED_STEM: f32 = 4.0;
/// Markers float this far above the ground, metres, clear of its triangles.
const MARK_LIFT: f32 = 0.1;
/// Marker lines' width, logical pixels.
const MARK_WIDTH: f32 = 3.0;

#[derive(Resource, Default)]
struct Pointer {
    /// The pointer, when it is over the world rather than the interface.
    cursor: Option<Vec2>,
    /// The ray from the eye through it.
    ray: Option<Ray3d>,
    /// Where that ray meets the ground, when a tool needs to know.
    ground: Option<Vec3>,
    /// Where the left button went down over the world.
    press: Option<Vec2>,
}

pub fn install(app: &mut App) {
    app.init_resource::<Pointer>()
        .add_systems(Startup, marker_style)
        .add_systems(Update, (aim, act, mark).chain());
}

/// Markers draw over the forest, so they are seen through foliage.
fn marker_style(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.0;
    config.line.width = MARK_WIDTH;
}

fn aim(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    ground: Query<(), With<GroundPart>>,
    mut contexts: EguiContexts,
    mut ray_cast: MeshRayCast,
    viewer: Res<Viewer>,
    mut pointer: ResMut<Pointer>,
) {
    let over_interface = contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.is_pointer_over_egui() || ctx.egui_wants_pointer_input());
    let cursor = windows
        .iter()
        .find_map(Window::cursor_position)
        .filter(|_| !over_interface && !viewer.menu.open);
    pointer.cursor = cursor;
    pointer.ray = cursor.and_then(|cursor| {
        cameras
            .iter()
            .find_map(|(camera, transform)| camera.viewport_to_world(transform, cursor).ok())
    });
    pointer.ground = None;
    if viewer.tool == Tool::Look {
        return;
    }
    let Some(ray) = pointer.ray else {
        return;
    };
    let on_ground = |entity| ground.contains(entity);
    let settings = MeshRayCastSettings::default().with_filter(&on_ground);
    pointer.ground = ray_cast
        .cast_ray(ray, &settings)
        .first()
        .map(|(_, hit)| hit.point);
}

/// A click with a tool in hand uses it.
fn act(buttons: Res<ButtonInput<MouseButton>>, mut pointer: ResMut<Pointer>, viewer: Res<Viewer>) {
    if buttons.just_pressed(MouseButton::Left) {
        pointer.press = pointer.cursor;
    }
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let (Some(pressed), Some(released)) = (pointer.press.take(), pointer.cursor) else {
        return;
    };
    if pressed.distance(released) > CLICK_SLACK {
        return;
    }
    match viewer.tool {
        Tool::Look => {}
        Tool::Sow(species) => {
            if let Some(ray) = pointer.ray {
                viewer.simulation.send(tools::sow(species, ray));
            }
        }
        Tool::Clear => {
            if let Some(spot) = pointer.ground {
                let radius = viewer.gap_radius();
                viewer.simulation.send(Command::Gap {
                    x: spot.x,
                    z: spot.z,
                    radius,
                });
            }
        }
    }
}

/// Mark where the tool would act: a light ring round one in the plant's
/// colour, and a stem, where a seed would go; a red ring round what clearing
/// would take.
fn mark(pointer: Res<Pointer>, viewer: Res<Viewer>, mut gizmos: Gizmos) {
    let Some(spot) = pointer.ground else {
        return;
    };
    let spot = spot + Vec3::Y * MARK_LIFT;
    let flat = Isometry3d::new(spot, Quat::from_rotation_arc(Vec3::Z, Vec3::Y));
    match viewer.tool {
        Tool::Look => {}
        Tool::Sow(_) => {
            let Some(species) = viewer.tool.sown(viewer.stats.species()) else {
                return;
            };
            let [r, g, b, _] = icons::species_color(species.leaf).to_array();
            let color = Color::srgb_u8(r, g, b);
            let [r, g, b, _] = theme::TEXT.to_array();
            let light = Color::srgb_u8(r, g, b);
            gizmos.circle(flat, SEED_RING, light);
            gizmos.circle(flat, SEED_RING * 0.7, color);
            gizmos.line(spot, spot + Vec3::Y * SEED_STEM, light);
        }
        Tool::Clear => {
            let [r, g, b, _] = theme::DANGER.to_array();
            let color = Color::srgb_u8(r, g, b);
            let radius = viewer.gap_radius();
            gizmos.circle(flat, radius, color);
            gizmos.circle(flat, radius * 0.96, color.with_alpha(0.4));
        }
    }
}
