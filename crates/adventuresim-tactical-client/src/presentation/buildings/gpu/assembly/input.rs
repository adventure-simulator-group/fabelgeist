//! Static building instances enter GPU assembly without temporary render entities.
use super::*;
use crate::presentation::buildings::{CompiledBuildingLevels, TacticalBuildingMaterials};
use adventuresim_building_generator::BuildingLodMaterial;

#[derive(Default, Resource)]
pub(in crate::presentation::buildings) struct PendingGpuBuildings {
    pub(super) parts: Vec<Part>,
    pub(super) buildings: Vec<Placement>,
}

pub(super) struct Placement {
    root: Entity,
    transform: Mat4,
    appearance: adventuresim_tactical_core::scene_input::DistantBuildingPlacement,
    compiled: Arc<CompiledBuildingLevels>,
}

impl PendingGpuBuildings {
    pub(in crate::presentation::buildings) fn clear(&mut self) {
        self.parts.clear();
        self.buildings.clear();
        READY.store(false, Ordering::Relaxed);
    }

    pub(in crate::presentation::buildings) fn push(
        &mut self,
        root: Entity,
        transform: &Transform,
        appearance: adventuresim_tactical_core::scene_input::DistantBuildingPlacement,
        compiled: &Arc<CompiledBuildingLevels>,
    ) {
        self.buildings.push(Placement {
            root,
            transform: transform.to_matrix(),
            appearance,
            compiled: compiled.clone(),
        });
    }

    pub(super) fn is_empty(&self) -> bool {
        self.parts.is_empty() && self.buildings.is_empty()
    }

    pub(super) fn iter<'a>(
        &'a self,
        materials: Option<&'a TacticalBuildingMaterials>,
    ) -> impl Iterator<Item = Part> + Clone + 'a {
        self.parts
            .iter()
            .cloned()
            .chain(self.buildings.iter().flat_map(move |placement| {
                let compiled = &placement.compiled;
                let palette = materials
                    .expect("city building materials")
                    .for_distant_building(
                        placement.appearance.prosperity,
                        placement.appearance.exterior_variant(),
                    );
                [&compiled.lod0, &compiled.lod1, &compiled.lod2]
                    .into_iter()
                    .enumerate()
                    .flat_map(move |(level, batches)| {
                        batches.iter().map(move |batch| Part {
                            entity: None,
                            root: placement.root,
                            transform: placement.transform,
                            local_transform: batch.transform,
                            uv_offset: batch.uv_offset,
                            mesh: batch.mesh.clone(),
                            material: palette.get(batch.material),
                            level: level as u32
                                | if batch.material == BuildingLodMaterial::FacadeDetails {
                                    FACADE_OVERLAY_FLAG
                                } else {
                                    0
                                },
                            fade: None,
                        })
                    })
            }))
    }
}
