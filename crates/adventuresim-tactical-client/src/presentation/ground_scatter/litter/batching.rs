//! Share static litter geometry; plants retain their original batch frame.

use super::*;
use anyhow::Result;

pub(super) use assembly::LitterPlacement;
pub(in crate::presentation) use shared::SharedLitterPlugin;
pub(in crate::presentation) use work::AcceptedLitterWork;

mod assembly;
mod shared;
mod work;

pub(super) fn append_litter_batch(
    source: &Handle<Mesh>,
    mut transform: Transform,
    cell_size: f32,
    batches: &mut BTreeMap<(i32, i32), LitterBatch>,
    kind: BatchKind,
) {
    let cell = (
        (transform.translation.x / cell_size).floor() as i32,
        (transform.translation.z / cell_size).floor() as i32,
    );
    transform.translation.x -= cell.0 as f32 * cell_size;
    transform.translation.z -= cell.1 as f32 * cell_size;
    let batch = batches.entry(cell).or_default();
    let slot = match kind {
        BatchKind::Leaves => &mut batch.leaves,
        BatchKind::Twigs => &mut batch.twigs,
        BatchKind::Plants => &mut batch.plants,
    };
    slot.push(LitterPlacement {
        source: source.clone(),
        frame: transform,
    });
}

pub(super) fn spawn_batches(
    commands: &mut Commands,
    meshes: &mut bevy::prelude::Assets<Mesh>,
    assets: &Assets,
    batches: BTreeMap<(i32, i32), LitterBatch>,
) -> Result<AcceptedLitterWork> {
    let work = work::record(meshes, &batches)?;
    for ((cell_x, cell_z), batch) in batches {
        let transform = Transform::from_xyz(
            cell_x as f32 * LITTER_BATCH_CELL_METRES,
            0.0,
            cell_z as f32 * LITTER_BATCH_CELL_METRES,
        );
        shared::SharedLitterCell {
            placements: &batch.leaves,
            frame: transform,
            layer: GroundScatterLayer::DryLeaves,
            range: batched_litter_visibility(DRY_LEAF_LOCAL_END_METRES),
            label: "Shared tactical dry leaves",
        }
        .spawn(commands, meshes, &assets.dry_leaf_material)?;
        shared::SharedLitterCell {
            placements: &batch.twigs,
            frame: transform,
            layer: GroundScatterLayer::Twigs,
            range: batched_litter_visibility(TWIG_LOCAL_END_METRES),
            label: "Shared tactical twigs",
        }
        .spawn(commands, meshes, &assets.twig_material)?;
        if let Some(mesh) = assembly::assemble(meshes, &batch.plants)? {
            commands.spawn((
                Name::new("Batched tactical woodland-floor plants"),
                GroundScatterLayer::Understory,
                NotShadowCaster,
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(assets.woodland_plant_material.clone()),
                batched_litter_visibility(WOODLAND_PLANT_LOCAL_END_METRES),
                transform,
            ));
        }
    }
    Ok(work)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;

    #[test]
    fn prototypes_survive_between_scenes_while_batches_upload_only() {
        let mut meshes = bevy::prelude::Assets::<Mesh>::default();
        for prototype in [
            dry_leaf_patch_mesh(0).unwrap(),
            twig_patch_mesh(0).unwrap(),
            woodland_plant_patch_mesh(0).unwrap(),
        ] {
            assert_eq!(prototype.asset_usage, RenderAssetUsages::MAIN_WORLD);
            let source = meshes.add(prototype);
            for _scene in 0..2 {
                let mut batches = BTreeMap::new();
                append_litter_batch(
                    &source,
                    Transform::IDENTITY,
                    LITTER_BATCH_CELL_METRES,
                    &mut batches,
                    BatchKind::Leaves,
                );
                let batch = assembly::assemble(&meshes, &batches[&(0, 0)].leaves)
                    .unwrap()
                    .unwrap();
                assert_eq!(batch.asset_usage, RenderAssetUsages::RENDER_WORLD);
                assert_eq!(
                    batch.count_vertices(),
                    meshes.get(&source).unwrap().count_vertices()
                );
            }
        }
    }
}
