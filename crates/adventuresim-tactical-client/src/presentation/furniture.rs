//! Outdoor recipes use the same mesh conversion and semantic materials as buildings.
use std::collections::BTreeMap;

use adventuresim_building_generator::{
    BuildingLodMaterial,
    furniture::{FurnitureKey, FurnitureKind},
};

use super::{recipe_mesh::recipe_mesh, *};

const SMALL_FURNITURE_FADE_METRES: std::ops::Range<f32> = 180.0..230.0;
const STALL_FADE_METRES: std::ops::Range<f32> = 350.0..450.0;

#[derive(Component)]
pub(crate) struct PresentedFurnitureMesh {
    pub(crate) material: BuildingLodMaterial,
}

pub(super) struct FurniturePresentationPlugin;

impl Plugin for FurniturePresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FurnitureMeshCache>()
            .add_observer(on_furniture_added)
            .add_observer(on_vista_furniture);
    }
}

#[derive(Default, Resource)]
struct FurnitureMeshCache(BTreeMap<FurnitureKey, Vec<FurnitureBatch>>);

struct FurnitureBatch {
    mesh: Handle<Mesh>,
    material: BuildingLodMaterial,
}

fn on_furniture_added(
    event: On<Add, SceneFurniture>,
    furniture: Query<&SceneFurniture>,
    mut commands: Commands,
    mut cache: ResMut<FurnitureMeshCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
) -> Result {
    let instance = furniture.get(event.entity)?;
    let batches = cache.0.entry(instance.key).or_insert_with(|| {
        instance
            .key
            .recipe()
            .meshes
            .iter()
            .map(|batch| FurnitureBatch {
                mesh: meshes.add(recipe_mesh(batch, Vec3::ZERO)),
                material: batch.material,
            })
            .collect()
    });
    let end_margin = match instance.key.kind {
        FurnitureKind::CanvasStall => STALL_FADE_METRES,
        _ => SMALL_FURNITURE_FADE_METRES,
    };
    commands
        .entity(event.entity)
        .insert(Visibility::default())
        .with_children(|parent| {
            for batch in batches.iter() {
                parent.spawn((
                    Name::new(format!("{:?} furniture", instance.key.kind)),
                    PresentedFurnitureMesh {
                        material: batch.material,
                    },
                    Mesh3d(batch.mesh.clone()),
                    MeshMaterial3d(materials.get_for_building(instance.id.0, batch.material)),
                    Transform::IDENTITY,
                    VisibilityRange {
                        start_margin: 0.0..0.0,
                        end_margin: end_margin.clone(),
                        use_aabb: false,
                    },
                ));
            }
        });
    Ok(())
}

#[derive(Component)]
struct VistaFurniturePresentation;
fn on_vista_furniture(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    existing: Query<Entity, With<VistaFurniturePresentation>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    for instance in &bundle.distant_furniture {
        commands.spawn((
            VistaFurniturePresentation,
            instance.scene,
            Transform::from_translation(instance.position_metres)
                .with_rotation(Quat::from_rotation_y(instance.orientation.yaw_radians())),
        ));
    }
}
