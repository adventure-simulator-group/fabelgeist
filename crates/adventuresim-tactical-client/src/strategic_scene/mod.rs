//! Retained strategic presentation in the tactical renderer's existing world.
//! No entity here participates in replication, movement, damage, or strategic state.
mod buildings;
mod cached_views;
mod equipment;
mod equipment_readiness;
mod instances;
mod lighting_diagnostics;
pub(crate) mod protocol;
mod render_diagnostics;
mod shadows;
mod staging;
pub(crate) mod status;
mod street;
mod views;
mod visibility;
mod world;

use crate::animation::StrategicModel;
use adventuresim_tactical_core::prelude::{CharacterId, SkeletonState};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use buildings::Venue;
use protocol::{PlaceId, PortraitId, StrategicView};
use std::collections::HashMap;
#[cfg(target_family = "wasm")]
pub(crate) use world::SceneDocument;

#[derive(Component)]
struct SceneRoot;

#[derive(Component)]
struct SceneModel(usize);

struct RetainedPerson {
    entity: Entity,
    anchor: Vec3,
    facing: Quat,
    layer: usize,
}

#[derive(Component)]
struct SceneMesh;

#[derive(Resource, Default)]
struct RetainedScene {
    location: String,
    digest: String,
    root: Option<Entity>,
    venues: HashMap<PlaceId, Venue>,
    street: Option<street::Street>,
    people: HashMap<PortraitId, RetainedPerson>,
    next_person_layer: usize,
    next_place: usize,
    error: Option<String>,
    pending: std::collections::VecDeque<adventuresim_tactical_core::prelude::GeneratedBuilding>,
}

#[cfg(test)]
mod tests;

pub(crate) struct StrategicScenePlugin;

impl Plugin for StrategicScenePlugin {
    fn build(&self, app: &mut App) {
        status::install(app);
        cached_views::install(app);
        visibility::install(app);
        app.init_resource::<RetainedScene>()
            .init_resource::<views::ViewCameras>()
            .add_systems(
                Update,
                (
                    world::retain_scene,
                    street::build_ground,
                    retain_people,
                    equipment::sync_equipment,
                    sync_room_lights,
                    views::sync_views,
                    shadows::sync_character_shadows,
                )
                    .chain()
                    .before(crate::equipment::visuals::EquipmentVisualSystems),
            )
            .add_systems(
                PostUpdate,
                (
                    inherit_model_layers.run_if(model_hierarchy_changed),
                    cull_equipment,
                )
                    .chain()
                    .before(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
            );
    }
}

fn retain_people(
    mut commands: Commands,
    requested: Option<Res<StrategicView>>,
    mut scene: ResMut<RetainedScene>,
) {
    let Some(view) = requested else {
        return;
    };
    if !view.is_changed() && !scene.is_changed() {
        return;
    }
    let Some(root) = scene.root else {
        return;
    };
    scene.people.retain(|id, person| {
        let keep = view.people.iter().any(|person| person.id == *id);
        if !keep {
            commands.entity(person.entity).despawn();
        }
        keep
    });
    for person in &view.people {
        let Some(venue) = scene.venues.get(&person.place) else {
            continue;
        };
        // Each resident has a stable position within their venue, shared by all views.
        let rank = view
            .people
            .iter()
            .filter(|other| {
                other.place == person.place && other.presentation == person.presentation
            })
            .position(|other| other.id == person.id)
            .unwrap_or(0);
        let position = venue.positions[rank % venue.positions.len()];
        let anchor = position.translation;
        let facing = position.rotation;
        if let Some(current) = scene.people.get_mut(&person.id) {
            if current.anchor != anchor || current.facing != facing {
                current.anchor = anchor;
                current.facing = facing;
                commands
                    .entity(current.entity)
                    .insert(model_transform(anchor).with_rotation(facing));
            }
            continue;
        }
        let layer = protocol::COMPOSITOR_LAYER + 1 + scene.next_person_layer;
        scene.next_person_layer += 1;
        let entity = commands
            .spawn((
                StrategicModel,
                SceneModel(layer),
                CharacterId::from(person.id.0),
                SkeletonState::default(),
                model_transform(anchor).with_rotation(facing),
                Visibility::Inherited,
                RenderLayers::none().with(layer),
                ChildOf(root),
            ))
            .id();
        scene.people.insert(
            person.id,
            RetainedPerson {
                entity,
                anchor,
                facing,
                layer,
            },
        );
    }
}

fn model_transform(anchor: Vec3) -> Transform {
    let height = adventuresim_tactical_core::combat_config::runtime_animation_config()
        .playback
        .player_visual_y_offset_metres;
    Transform::from_translation(anchor - Vec3::Y * height)
}

#[expect(
    clippy::type_complexity,
    reason = "structural changes that can introduce a character mesh owner"
)]
fn model_hierarchy_changed(
    changes: Query<
        (),
        Or<(
            Added<Mesh3d>,
            Changed<ChildOf>,
            Changed<SceneModel>,
            Changed<crate::equipment::ItemPlaceholder>,
            Changed<adventuresim_tactical_core::prelude::ItemOf>,
        )>,
    >,
) -> bool {
    !changes.is_empty()
}

fn inherit_model_layers(
    mut commands: Commands,
    added: Query<Entity, (With<Mesh3d>, Without<RenderLayers>)>,
    parents: Query<&ChildOf>,
    models: Query<&SceneModel>,
    equipment_roots: Query<&crate::equipment::ItemPlaceholder>,
    owners: Query<&adventuresim_tactical_core::prelude::ItemOf>,
) {
    for entity in &added {
        let mut current = entity;
        while let Ok(parent) = parents.get(current) {
            current = parent.parent();
            let equipment_owner = equipment_roots
                .get(current)
                .ok()
                .and_then(|item| owners.get(item.0).ok())
                .map(|owner| owner.0);
            if let Ok(model) = models
                .get(current)
                .or_else(|_| models.get(equipment_owner.unwrap_or(current)))
            {
                commands
                    .entity(entity)
                    .insert((RenderLayers::none().with(model.0), SceneMesh));
                break;
            }
        }
    }
}

#[expect(
    clippy::type_complexity,
    reason = "Bevy query selects strategic meshes that need initial culling bounds"
)]
fn cull_equipment(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    equipment: Query<
        (Entity, &Mesh3d),
        (
            With<SceneMesh>,
            With<bevy::camera::visibility::NoFrustumCulling>,
        ),
    >,
) {
    // Strategic actors only idle in place. A conservative margin covers the
    // idle motion and identity morphs while excluding other venues' equipment.
    const IDLE_BOUNDS_MARGIN_METRES: f32 = 0.4;
    use bevy::camera::{primitives::MeshAabb, visibility::NoAutoAabb};
    for (entity, mesh) in &equipment {
        let Some(mut bounds) = meshes.get(&mesh.0).and_then(MeshAabb::compute_aabb) else {
            continue;
        };
        bounds.half_extents += bevy::math::Vec3A::splat(IDLE_BOUNDS_MARGIN_METRES);
        commands
            .entity(entity)
            .insert((bounds, NoAutoAabb))
            .remove::<bevy::camera::visibility::NoFrustumCulling>();
    }
}

#[expect(
    clippy::type_complexity,
    reason = "Bevy query selects both light types used by strategic rooms"
)]
fn sync_room_lights(
    mut commands: Commands,
    scene: Res<RetainedScene>,
    lights: Query<(Entity, Option<&RenderLayers>), Or<(With<DirectionalLight>, With<PointLight>)>>,
) {
    let layers = RenderLayers::from_iter(
        std::iter::once(buildings::ROOM_LAYER)
            .chain(scene.people.values().map(|person| person.layer)),
    );
    for (entity, current) in &lights {
        if current.is_none_or(|current| current.iter().any(|layer| layer == buildings::ROOM_LAYER))
            && current != Some(&layers)
        {
            commands.entity(entity).insert(layers.clone());
        }
    }
}

/// End local display ownership before replicated tactical entities are admitted.
pub(crate) fn release_scene(world: &mut World) {
    let root = world.resource::<RetainedScene>().root;
    if let Some(root) = root
        && let Ok(entity) = world.get_entity_mut(root)
    {
        entity.despawn();
    }
    world.insert_resource(RetainedScene::default());
    crate::presentation::clear_scene_entities(world);
}
