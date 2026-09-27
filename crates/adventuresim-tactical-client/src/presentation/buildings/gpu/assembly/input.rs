//! Static building instances enter GPU assembly without temporary render entities.
use super::*;
use crate::presentation::buildings::{CompiledBuildingLevels, TacticalBuildingMaterials};
use adventuresim_building_generator::BuildingLodMaterial;

#[derive(Default, Resource)]
pub(in crate::presentation::buildings) struct PendingGpuBuildings {
    pub(super) parts: Vec<Part>,
}

impl PendingGpuBuildings {
    pub(in crate::presentation::buildings) fn clear(&mut self) {
        self.parts.clear();
        READY.store(false, Ordering::Relaxed);
    }

    pub(in crate::presentation::buildings) fn push(
        &mut self,
        root: Entity,
        transform: &Transform,
        building_id: u64,
        compiled: &CompiledBuildingLevels,
        materials: &TacticalBuildingMaterials,
    ) {
        for (level, batches) in [&compiled.lod0, &compiled.lod1, &compiled.lod2]
            .into_iter()
            .enumerate()
        {
            self.parts.extend(batches.iter().map(|batch| Part {
                entity: None,
                root,
                transform: transform.to_matrix(),
                mesh: batch.mesh.clone(),
                material: materials.get_for_building(building_id, batch.material),
                level: level as u32
                    | if batch.material == BuildingLodMaterial::FacadeDetails {
                        FACADE_OVERLAY_FLAG
                    } else {
                        0
                    },
                fade: None,
            }));
        }
    }
}
