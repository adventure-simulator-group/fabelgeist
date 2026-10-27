//! Shared accepted enclosure cells retain material and spatial batching.
use super::*;
use adventuresim_tactical_core::prelude::SceneBoundary;
use batching::{BoundaryBatches, BoundaryPlacement};
use projection::{PendingDistantBoundaries, on_vista, project_pending};

mod batching;
mod projection;

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
    batches.insert_boundary(boundary, BoundaryPlacement::GateRelative, &materials)?;
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
