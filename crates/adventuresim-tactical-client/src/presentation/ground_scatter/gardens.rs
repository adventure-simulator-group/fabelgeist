//! Render the accepted property specimen without sampling, scaling or pruning it.
use super::*;
use adventuresim_tactical_core::{
    city_layout::{GardenPlantId, GardenSpecimen},
    prelude::SceneGarden,
};
mod projection;
use projection::{PendingDistantGardens, on_vista, project_pending};

pub(in crate::presentation) struct GardenPresentationPlugin;
impl Plugin for GardenPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingDistantGardens>()
            .add_observer(on_garden)
            .add_observer(on_vista)
            .add_systems(Update, project_pending);
    }
}
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
    let garden = gardens.get(event.entity)?;
    assets.prepare();
    commands
        .entity(event.entity)
        .insert(Visibility::default())
        .with_children(|parent| assets.spawn(parent, garden));
    Ok(())
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

    fn spawn(&self, parent: &mut ChildSpawnerCommands, scene: &SceneGarden) {
        let garden = scene.garden();
        for (plant, support) in garden.plants.iter().zip(scene.plant_support()) {
            let presentation = match plant.specimen {
                GardenSpecimen::CommonHazel => &self.cache.hazel,
            };
            let transform = Transform::from_xyz(
                plant.centre_metres.metres().x,
                support.elevation.metres(),
                plant.centre_metres.metres().y,
            )
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
