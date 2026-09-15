//! Keep reusable litter prototypes on the CPU and upload only assembled batches.

use super::*;

pub(super) fn append_litter_batch(
    meshes: &bevy::prelude::Assets<Mesh>,
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
    let Some(source) = meshes.get(source) else {
        return;
    };
    let mut transformed = source.clone().transformed_by(transform);
    transformed.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let batch = batches.entry(cell).or_default();
    let slot = match kind {
        BatchKind::Leaves => &mut batch.leaves,
        BatchKind::Twigs => &mut batch.twigs,
        BatchKind::Plants => &mut batch.plants,
    };
    if let Some(batch) = slot {
        batch
            .merge(&transformed)
            .expect("litter variants share one vertex contract");
    } else {
        *slot = Some(transformed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prototypes_survive_between_scenes_while_batches_upload_only() {
        let mut meshes = bevy::prelude::Assets::<Mesh>::default();
        for prototype in [
            dry_leaf_patch_mesh(0),
            twig_patch_mesh(0),
            woodland_plant_patch_mesh(0),
        ] {
            assert_eq!(prototype.asset_usage, RenderAssetUsages::MAIN_WORLD);
            let source = meshes.add(prototype);
            for _scene in 0..2 {
                let mut batches = BTreeMap::new();
                append_litter_batch(
                    &meshes,
                    &source,
                    Transform::IDENTITY,
                    LITTER_BATCH_CELL_METRES,
                    &mut batches,
                    BatchKind::Leaves,
                );
                let batch = batches[&(0, 0)].leaves.as_ref().unwrap();
                assert_eq!(batch.asset_usage, RenderAssetUsages::RENDER_WORLD);
                assert_eq!(
                    batch.count_vertices(),
                    meshes.get(&source).unwrap().count_vertices()
                );
            }
        }
    }
}
