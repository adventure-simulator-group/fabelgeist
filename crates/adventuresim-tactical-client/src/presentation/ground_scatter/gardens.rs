//! Render the accepted property specimen without sampling, scaling or pruning it.
use super::*;
use adventuresim_tactical_core::{
    city_layout::{CityGarden, GardenPlantId, GardenSpecimen},
    prelude::SceneGarden,
};
use adventuresim_tactical_netcode::message::SceneVistaBundle;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Component)]
pub(crate) struct ManagedGardenPlant(pub GardenPlantId);
#[derive(Component)]
pub(in crate::presentation) struct DistantGardenPresentation;

#[derive(SystemParam)]
pub(in crate::presentation) struct GardenAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    leaf_materials: ResMut<'w, Assets<TacticalTreeLeafCardMaterial>>,
    cache: ResMut<'w, WoodyUnderstoryPresentationCache>,
    textures: Res<'w, ProceduralTextureAssets>,
}

pub(in crate::presentation) fn on_garden(
    event: On<Add, SceneGarden>,
    mut commands: Commands,
    gardens: Query<&SceneGarden>,
    mut assets: GardenAssets,
) -> Result {
    let garden = &gardens.get(event.entity)?.garden;
    assets.prepare();
    commands
        .entity(event.entity)
        .insert(Visibility::default())
        .with_children(|parent| assets.spawn(parent, garden));
    Ok(())
}

pub(in crate::presentation) fn on_vista(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    existing: Query<Entity, With<DistantGardenPresentation>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    for garden in &bundle.gardens {
        let Some(front) = bundle
            .distant_buildings
            .iter()
            .find(|b| b.id == garden.front_building_id)
        else {
            continue;
        };
        commands.spawn((
            DistantGardenPresentation,
            SceneGarden {
                garden: garden.clone(),
            },
            Transform::from_xyz(0.0, front.base_elevation_metres, 0.0),
        ));
    }
}

impl GardenAssets<'_> {
    fn prepare(&mut self) {
        ensure_understory_presentations(
            &mut self.meshes,
            &mut self.materials,
            &mut self.leaf_materials,
            &mut self.cache,
            &self.textures,
        );
    }

    fn spawn(&self, parent: &mut ChildSpawnerCommands, garden: &CityGarden) {
        for plant in &garden.plants {
            let presentation = match plant.specimen {
                GardenSpecimen::CommonHazel => &self.cache.hazel,
            };
            let transform = Transform::from_xyz(plant.centre_metres.x, 0.0, plant.centre_metres.y)
                .with_rotation(Quat::from_rotation_y(plant.orientation.yaw_radians()))
                .with_scale(Vec3::splat(plant.scale.value()));
            parent
                .spawn((
                    Name::new(format!("Property {} hazel {}", garden.owner.0, plant.id.0)),
                    ManagedGardenPlant(plant.id),
                    transform,
                    Visibility::default(),
                ))
                .with_children(|plant| {
                    plant.spawn((
                        Mesh3d(
                            presentation
                                .branches
                                .as_ref()
                                .expect("prepared garden branches")
                                .clone(),
                        ),
                        MeshMaterial3d(
                            presentation
                                .bark
                                .as_ref()
                                .expect("prepared garden bark")
                                .clone(),
                        ),
                    ));
                    plant.spawn((
                        TreeLeafRepresentation::AlphaCard,
                        Mesh3d(
                            presentation
                                .leaf_cards
                                .as_ref()
                                .expect("prepared garden leaves")
                                .clone(),
                        ),
                        MeshMaterial3d(
                            presentation
                                .leaves
                                .as_ref()
                                .expect("prepared garden leaf material")
                                .clone(),
                        ),
                    ));
                });
        }
    }
}
