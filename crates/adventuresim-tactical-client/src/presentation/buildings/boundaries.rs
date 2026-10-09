//! Shared accepted enclosure cells retain material and spatial batching.
use super::*;
use adventuresim_tactical_core::prelude::{CityBoundaryMaterial, SceneBoundary};
mod batching;
mod projection;
use batching::BoundaryBatches;
use projection::{PendingDistantBoundaries, on_vista, project_pending};

pub(super) struct BoundaryPresentationPlugin;
impl Plugin for BoundaryPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingDistantBoundaries>()
            .add_observer(on_boundary)
            .add_observer(on_vista)
            .add_systems(Update, project_pending);
    }
}

fn on_boundary(
    event: On<Add, SceneBoundary>,
    mut commands: Commands,
    boundaries: Query<&SceneBoundary>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
) -> Result {
    let boundary = boundaries.get(event.entity)?;
    let mut batches = BoundaryBatches::default();
    fixed(&mut batches, boundary, 0.0, &materials);
    commands
        .entity(event.entity)
        .insert(Visibility::default())
        .with_children(|parent| {
            for batch in batches.finish() {
                parent.spawn((
                    Mesh3d(meshes.add(batch.mesh)),
                    MeshMaterial3d(batch.material),
                    Transform::from_translation(batch.origin),
                ));
            }
        });
    Ok(())
}

fn fixed(
    batches: &mut BoundaryBatches,
    boundary: &SceneBoundary,
    elevation: f32,
    materials: &TacticalBuildingMaterials,
) {
    for cell in boundary.fixed_support().cells() {
        let material = match cell.material() {
            CityBoundaryMaterial::Masonry => BuildingLodMaterial::Wall(
                adventuresim_building_generator::WallMaterialClass::RubbleMasonry,
            ),
            CityBoundaryMaterial::Timber => BuildingLodMaterial::Timber,
            CityBoundaryMaterial::Iron => BuildingLodMaterial::Iron,
        };
        batches.insert(
            cell,
            elevation,
            materials
                .for_building(boundary.front_building_id().0)
                .get(material),
        );
    }
}
