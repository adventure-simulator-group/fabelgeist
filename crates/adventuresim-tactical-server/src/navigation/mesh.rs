//! Server-owned navigation mesh, built from authoritative collision geometry.

use super::*;
use avian_rerecast::ColliderToTriMesh;
use bevy::asset::AssetPlugin;
use bevy::ecs::change_detection::Tick;
use bevy_landmass::prelude::*;
use bevy_rerecast::{NavmeshPlugins, prelude::*, rerecast::TriMesh};
use landmass_rerecast::{Island3dBundle, LandmassRerecastPlugin, NavMeshHandle3d};
use std::collections::BTreeMap;

const GEOMETRY_POSITION_CHANGE_METRES: f32 = 0.05;
const GEOMETRY_ROTATION_CHANGE_RADIANS: f32 = 5.0_f32.to_radians();
const MESH_BUILD_TIMEOUT_SECONDS: f64 = 30.0;

#[derive(Clone, Copy)]
struct ColliderSnapshot {
    position: Vec3,
    rotation: Quat,
    shape_revision: Tick,
}

/// The mesh handle and Landmass world belong to this transient tactical scene.
#[derive(Resource, Default)]
pub(crate) struct TacticalNavigation {
    pub(crate) archipelago: Option<Entity>,
    pub(crate) mesh: Option<Handle<Navmesh>>,
    pub(crate) ready: bool,
    pub(crate) failed: bool,
    island: Option<Entity>,
    generated: bool,
    geometry: BTreeMap<Entity, ColliderSnapshot>,
    last_request: Option<f64>,
}

pub(crate) struct TacticalNavigationPlugin;

impl Plugin for TacticalNavigationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AssetPlugin>() {
            app.add_plugins(AssetPlugin::default());
        }
        app.add_plugins((
            NavmeshPlugins::default(),
            Landmass3dPlugin::default().in_schedule(Update),
            LandmassRerecastPlugin::default().in_schedule(Update),
        ))
        .set_navmesh_backend(collision_geometry)
        .init_resource::<TacticalNavigation>()
        .add_observer(mesh_ready)
        .add_systems(
            Update,
            (initialize_mesh, refresh_mesh)
                .chain()
                .before(LandmassSystems::SyncExistence),
        )
        .add_systems(
            Update,
            activate_mesh
                .after(LandmassSystems::SyncExistence)
                .before(crate::control::ControlSet::Navigate),
        );
    }
}

fn collision_geometry(
    input: In<NavmeshSettings>,
    colliders: Query<(
        Entity,
        &Collider,
        &adventuresim_tactical_core::avian3d::prelude::Position,
        &Rotation,
        &ColliderOf,
    )>,
    bodies: Query<(&RigidBody, Has<SceneDoor>, Has<SceneWindow>)>,
) -> TriMesh {
    colliders
        .iter()
        .filter_map(|(entity, collider, position, rotation, owner)| {
            if input
                .filter
                .as_ref()
                .is_some_and(|filter| !filter.contains(&entity))
            {
                return None;
            }
            let (body, door, window) = bodies.get(owner.body).ok()?;
            if !body.is_static() && !door && !window {
                return None;
            }
            collider.to_trimesh(*position, *rotation, 10)
        })
        .fold(TriMesh::default(), |mut mesh, collider| {
            mesh.extend(collider);
            mesh
        })
}

fn initialize_mesh(
    mut commands: Commands,
    mut navigation: ResMut<TacticalNavigation>,
    mut generator: NavmeshGenerator,
    config: Res<TacticalCombatConfig>,
    time: Res<Time>,
    static_colliders: Query<(&Collider, &ColliderOf)>,
    bodies: Query<&RigidBody>,
) {
    if navigation.mesh.is_some()
        || !static_colliders
            .iter()
            .any(|(_, owner)| bodies.get(owner.body).is_ok_and(RigidBody::is_static))
    {
        return;
    }
    let radius = adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES;
    let settings = navmesh_settings(&config);
    // Rerecast's polygon and detail surfaces can each be one voxel above the
    // physical surface. Sample around the feet, not the body's centre.
    let vertical_tolerance = config.ai.control.support_tolerance_metres.metres()
        + 2.0 * settings.agent_radius / settings.cell_height_fraction;
    let mut options: ArchipelagoOptions<bevy_landmass::coords::ThreeD> =
        ArchipelagoOptions::from_agent_radius(radius);
    options.point_sample_distance.distance_below = vertical_tolerance;
    options.point_sample_distance.distance_above = vertical_tolerance;
    options.point_sample_distance.horizontal_distance = radius;
    let archipelago = commands.spawn(Archipelago3d::new(options)).id();
    let handle = generator.generate(settings);
    let island = commands
        .spawn((
            Island3dBundle {
                island: Island,
                archipelago_ref: ArchipelagoRef3d::new(archipelago),
                nav_mesh: NavMeshHandle3d(handle.clone()),
            },
            Transform::default(),
        ))
        .id();
    navigation.archipelago = Some(archipelago);
    navigation.mesh = Some(handle);
    navigation.island = Some(island);
    navigation.last_request = Some(time.elapsed_secs_f64());
}

fn navmesh_settings(config: &TacticalCombatConfig) -> NavmeshSettings {
    let radius = adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES
        + config.ai.control.clearance_margin_metres.metres();
    let mut settings =
        NavmeshSettings::from_agent_3d(radius, CharacterDimensions::default().body_height_metres);
    settings.walkable_climb = config.movement.motor.maximum_step_height_metres;
    settings.walkable_slope_angle = config
        .movement
        .motor
        .maximum_walkable_slope_degrees
        .to_radians();
    settings
}

fn refresh_mesh(
    mut navigation: ResMut<TacticalNavigation>,
    mut generator: NavmeshGenerator,
    config: Res<TacticalCombatConfig>,
    time: Res<Time>,
    colliders: Query<(
        Entity,
        &ColliderOf,
        Ref<Collider>,
        &adventuresim_tactical_core::avian3d::prelude::Position,
        &Rotation,
    )>,
    bodies: Query<(&RigidBody, Has<SceneDoor>, Has<SceneWindow>)>,
) {
    let Some(handle) = navigation.mesh.clone() else {
        return;
    };
    if !navigation.ready
        && navigation
            .last_request
            .is_some_and(|last| time.elapsed_secs_f64() - last >= MESH_BUILD_TIMEOUT_SECONDS)
    {
        navigation.failed = true;
    }
    let current: BTreeMap<_, _> = colliders
        .iter()
        .filter_map(|(entity, owner, collider, position, rotation)| {
            let (body, door, window) = bodies.get(owner.body).ok()?;
            if door || window || body.is_static() {
                Some((
                    entity,
                    ColliderSnapshot {
                        position: position.0,
                        rotation: rotation.0,
                        shape_revision: collider.last_changed(),
                    },
                ))
            } else {
                None
            }
        })
        .collect();
    let changed = current.len() != navigation.geometry.len()
        || current.iter().any(|(entity, snapshot)| {
            navigation.geometry.get(entity).is_none_or(|old| {
                old.position.distance(snapshot.position) >= GEOMETRY_POSITION_CHANGE_METRES
                    || old.rotation.angle_between(snapshot.rotation)
                        >= GEOMETRY_ROTATION_CHANGE_RADIANS
                    || old.shape_revision != snapshot.shape_revision
            })
        });
    if !changed {
        return;
    }
    let now = time.elapsed_secs_f64();
    if navigation.last_request.is_some_and(|last| {
        now - last < f64::from(config.ai.control.mesh_rebuild_interval_seconds.seconds())
    }) {
        return;
    }
    if generator.regenerate(&handle, navmesh_settings(&config)) {
        navigation.geometry = current;
        navigation.last_request = Some(now);
        navigation.ready = false;
        navigation.failed = false;
        navigation.generated = false;
    }
}

fn mesh_ready(event: On<NavmeshReady>, mut navigation: ResMut<TacticalNavigation>) {
    if navigation
        .mesh
        .as_ref()
        .is_some_and(|mesh| mesh.id() == event.0)
    {
        navigation.generated = true;
        navigation.failed = false;
    }
}

fn activate_mesh(
    mut navigation: ResMut<TacticalNavigation>,
    islands: Query<&bevy_landmass::NavMeshHandle3d>,
    meshes: Res<Assets<bevy_landmass::NavMesh3d>>,
) {
    if !navigation.generated || navigation.ready {
        return;
    }
    if navigation
        .island
        .and_then(|island| islands.get(island).ok())
        .is_some_and(|handle| meshes.get(&handle.0).is_some())
    {
        navigation.ready = true;
    }
}
