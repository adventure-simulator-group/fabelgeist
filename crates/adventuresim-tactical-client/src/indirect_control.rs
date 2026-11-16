//! Contextual orders affect server-owned goals; camera motion stays local.

use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::{control::*, prelude::*};
use adventuresim_tactical_netcode::{bevy_replicon::prelude::ClientTriggerExt, control::*};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    transform::TransformSystems,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use bindings::{IndirectBindings, key_label, mouse_label};
use terrain::{TerrainPicking, update_terrain_picking};

mod bindings;
mod markers;
mod terrain;

use markers::{update_marker, update_selection_marker, update_waypoint_markers};

// Keyboard pan uses scene metres per second; wheel units have distinct scales.
const CAMERA_PAN_METRES_PER_SECOND: f32 = 8.0;
const CAMERA_SCROLL_METRES_PER_LINE: f32 = 0.5;
const CAMERA_SCROLL_METRES_PER_PIXEL: f32 = 0.01;

use crate::{
    camera::{CameraMode, TacticalCameraSet},
    player::{AttackState, BufferedMeleeAttack, ClientPlayer},
    presentation::TacticalGameplayCamera,
};

pub(crate) struct IndirectControlPlugin;

#[derive(Default)]
struct OrbitOrientation(Option<Quat>);

#[derive(Resource, Default)]
struct IndirectCameraOffset(Vec3);

#[derive(Resource, Default)]
struct SelectedCharacter(Option<Entity>);

type ChangedLocalControl<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Ref<'static, CharacterControl>,
        Ref<'static, ClientPlayer>,
    ),
    (
        With<ClientPlayer>,
        Or<(Changed<CharacterControl>, Added<ClientPlayer>)>,
    ),
>;

impl Plugin for IndirectControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalControl>()
            .init_resource::<IndirectBindings>()
            .register_type::<IndirectBindings>()
            .init_resource::<TerrainPicking>()
            .init_resource::<IndirectCameraOffset>()
            .init_resource::<SelectedCharacter>()
            .add_systems(
                Update,
                (
                    synchronize_authority,
                    update_terrain_picking,
                    issue_orders,
                    show_status,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, orbit_camera.before(TacticalCameraSet::Offset))
            .add_systems(
                PostUpdate,
                pan_camera
                    .after(TacticalCameraSet::Offset)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                (
                    update_marker,
                    update_selection_marker,
                    update_waypoint_markers,
                )
                    .after(TransformSystems::Propagate),
            );
    }
}

fn synchronize_authority(
    mut commands: Commands,
    mut local: ResMut<LocalControl>,
    players: ChangedLocalControl,
) {
    for (entity, control, player) in &players {
        if player.is_added() || local.acknowledged != *control {
            local.acknowledged = *control;
            local.pending = None;
            commands
                .entity(entity)
                .remove::<(AttackState, BufferedMeleeAttack)>();
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct OrderPicking<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    cameras:
        Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<TacticalGameplayCamera>>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    owners: Query<'w, 's, &'static ColliderOf>,
    targets: Query<'w, 's, &'static TacticalCombatSide>,
    config: Res<'w, TacticalCombatConfig>,
    terrain: Res<'w, TerrainPicking>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy provides input, selection, authority, and picking state independently"
)]
fn issue_orders(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<IndirectBindings>,
    mut local: ResMut<LocalControl>,
    mut selected: ResMut<SelectedCharacter>,
    mut cancelled: Local<bool>,
    players: Query<
        (
            Entity,
            &Transform,
            &CharacterDimensions,
            &TacticalCombatSide,
            &CharacterOrder,
        ),
        With<ClientPlayer>,
    >,
    picking: OrderPicking,
) {
    if !keys.pressed(bindings.order_modifier) {
        *cancelled = false;
    }
    if keys.just_pressed(bindings.cancel) && keys.pressed(bindings.order_modifier) {
        *cancelled = true;
    }
    let Some((entity, transform, dimensions, side, order)) = players.iter().next() else {
        return;
    };
    if local.pending.is_some() {
        return;
    }
    // Reconnects retain the server's monotonic order sequence.
    local.sequence = local.sequence.max(order.sequence);
    let change = if keys.just_pressed(bindings.toggle) {
        Some(ControlChange::SetMode(match local.acknowledged.mode {
            ControlMode::Direct => ControlMode::Indirect,
            ControlMode::Indirect => ControlMode::Direct,
        }))
    } else if local.acknowledged.mode != ControlMode::Indirect {
        None
    } else if keys.just_pressed(bindings.stop) {
        Some(ControlChange::Stop)
    } else if keys.just_pressed(bindings.hold) {
        Position::from_metres(transform.translation - Vec3::Y * dimensions.body_height_metres * 0.5)
            .ok()
            .map(|anchor| ControlChange::Order(CharacterGoal::Hold { anchor }))
    } else if (mouse.just_pressed(bindings.order)
        || (keys.pressed(bindings.order_modifier)
            && !*cancelled
            && mouse.just_pressed(bindings.select)))
        && !keys.any_pressed(bindings.orbit_modifiers)
    {
        let waypoint = keys.any_pressed(bindings.waypoint_modifiers);
        picking.order(entity, *side, waypoint)
    } else {
        if mouse.just_pressed(bindings.select)
            && !keys.pressed(bindings.order_modifier)
            && !keys.any_pressed(bindings.selection_modifiers)
            && !keys.any_pressed(bindings.waypoint_modifiers)
            && !keys.any_pressed(bindings.orbit_modifiers)
        {
            selected.0 = picking.character();
        }
        None
    };
    if let Some(change) = change {
        commands
            .entity(entity)
            .remove::<(AttackState, BufferedMeleeAttack)>();
        commands.client_trigger(local.request(change));
    }
}

impl OrderPicking<'_, '_> {
    fn ray(&self) -> Option<Ray3d> {
        let (camera, transform) = self.cameras.iter().next()?;
        let window = self.windows.iter().next()?;
        let cursor = window.cursor_position()?;
        camera.viewport_to_world(transform, cursor).ok()
    }

    fn character(&self) -> Option<Entity> {
        let ray = self.ray()?;
        let hit = self.spatial.cast_ray(
            ray.origin,
            ray.direction,
            self.config.ai.control.click_distance_metres.metres(),
            true,
            &SpatialQueryFilter::default(),
        )?;
        let entity = self
            .owners
            .get(hit.entity)
            .map_or(hit.entity, |owner| owner.body);
        self.targets.get(entity).ok().map(|_| entity)
    }

    fn order(
        &self,
        actor: Entity,
        side: TacticalCombatSide,
        waypoint: bool,
    ) -> Option<ControlChange> {
        let ray = self.ray()?;
        let maximum = self.config.ai.control.click_distance_metres.metres();
        let terrain = self.terrain.distance(ray.origin, ray.direction, maximum);
        let hit = self.spatial.cast_ray(
            ray.origin,
            ray.direction,
            maximum,
            true,
            &SpatialQueryFilter::from_excluded_entities([actor]),
        );
        if !waypoint
            && let Some(hit) = hit
            && terrain.is_none_or(|distance| hit.distance < distance)
        {
            let target = self
                .owners
                .get(hit.entity)
                .map_or(hit.entity, |owner| owner.body);
            if self
                .targets
                .get(target)
                .is_ok_and(|target_side| *target_side != side)
            {
                return Some(ControlChange::Order(CharacterGoal::Attack { target }));
            }
        }
        let distance = terrain?;
        let position = Position::from_metres(ray.origin + ray.direction * distance).ok()?;
        let destination = NavigationDestination { position };
        Some(if waypoint {
            ControlChange::AppendWaypoint(destination)
        } else {
            ControlChange::Order(CharacterGoal::MoveTo(destination))
        })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy supplies camera, cursor, input, and acknowledged control separately"
)]
fn orbit_camera(
    local: Res<LocalControl>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<IndirectBindings>,
    motion: Res<AccumulatedMouseMotion>,
    config: Res<TacticalCombatConfig>,
    mut mode: ResMut<CameraMode>,
    mut orientation: Local<OrbitOrientation>,
    mut cameras: Query<&mut Transform, With<TacticalGameplayCamera>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if local.acknowledged.mode != ControlMode::Indirect {
        orientation.0 = None;
        return;
    }
    mode.third_person = true;
    for mut cursor in &mut cursors {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
    for mut camera in &mut cameras {
        let rotation = orientation.0.get_or_insert(camera.rotation);
        if mouse.pressed(bindings.orbit)
            || (mouse.pressed(bindings.select) && keys.any_pressed(bindings.orbit_modifiers))
        {
            let (yaw, pitch, _) = rotation.to_euler(EulerRot::YXZ);
            let sensitivity = config
                .ai
                .control
                .orbit_mouse_degrees_per_pixel
                .degrees_per_pixel()
                .to_radians();
            // The open hemisphere avoids an inverted orbit and the look-at pole.
            let pitch = (pitch - motion.delta.y * sensitivity).clamp(
                -std::f32::consts::FRAC_PI_2.next_down(),
                std::f32::consts::FRAC_PI_2.next_down(),
            );
            *rotation = Quat::from_euler(
                EulerRot::YXZ,
                yaw - motion.delta.x * sensitivity,
                pitch,
                0.0,
            );
        }
        camera.rotation = *rotation;
    }
}

fn pan_camera(
    local: Res<LocalControl>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<IndirectBindings>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    mut offset: ResMut<IndirectCameraOffset>,
    mut cameras: Query<&mut Transform, With<TacticalGameplayCamera>>,
) {
    if local.acknowledged.mode != ControlMode::Indirect {
        offset.0 = Vec3::ZERO;
        return;
    }
    for mut camera in &mut cameras {
        let forward = camera.forward().as_vec3().with_y(0.0).normalize_or_zero();
        let right = camera.right().as_vec3().with_y(0.0).normalize_or_zero();
        let axis = Vec2::new(
            i8::from(keys.pressed(bindings.pan_right)) as f32
                - i8::from(keys.pressed(bindings.pan_left)) as f32,
            i8::from(keys.pressed(bindings.pan_forward)) as f32
                - i8::from(keys.pressed(bindings.pan_backward)) as f32,
        )
        .clamp_length_max(1.0);
        offset.0 +=
            (right * axis.x + forward * axis.y) * CAMERA_PAN_METRES_PER_SECOND * time.delta_secs();
        let scale = match scroll.unit {
            MouseScrollUnit::Line => CAMERA_SCROLL_METRES_PER_LINE,
            MouseScrollUnit::Pixel => CAMERA_SCROLL_METRES_PER_PIXEL,
        };
        offset.0.y += scroll.delta.y * scale;
        camera.translation += offset.0;
    }
}

fn show_status(
    local: Res<LocalControl>,
    bindings: Res<IndirectBindings>,
    orders: Query<&CharacterOrder, With<ClientPlayer>>,
    mut last: Local<crate::hud::LastPushed<String>>,
) {
    let label = if local.pending.is_some() {
        "Changing control…".to_owned()
    } else if let Some(error) = local.rejection {
        error.to_string()
    } else if local.is_direct() {
        format!("Direct control · {}: indirect", key_label(bindings.toggle))
    } else {
        let status = orders
            .iter()
            .next()
            .map(|order| match order.status {
                OrderStatus::Idle => "Idle".to_owned(),
                OrderStatus::Executing => "Following order".to_owned(),
                OrderStatus::Defending => "Defending".to_owned(),
                OrderStatus::Completed => "Arrived".to_owned(),
                OrderStatus::Failed(error) => error.to_string(),
            })
            .unwrap_or_default();
        format!(
            "Indirect · {status} · {}+{}: order · {}+order: waypoint · {}+drag: orbit · {}: stop · {}: hold",
            key_label(bindings.order_modifier),
            mouse_label(bindings.select),
            key_label(bindings.waypoint_modifiers[0]),
            key_label(bindings.orbit_modifiers[0]),
            key_label(bindings.stop),
            key_label(bindings.hold)
        )
    };
    if let Some(label) = last.update(label) {
        crate::hud::text(crate::hud::HudText::Control, label);
    }
}
