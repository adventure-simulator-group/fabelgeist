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

    pub(super) fn groups(&self, materials: Option<&TacticalBuildingMaterials>) -> Vec<Group> {
        let mut groups = Group::from_parts(self.parts.iter().cloned());
        let mut prototypes = HashMap::new();
        for placement in &self.buildings {
            let palette = materials
                .expect("city building materials")
                .for_distant_building(
                    placement.appearance.prosperity,
                    placement.appearance.exterior_variant(),
                );
            // The palette's infill is unique to each appearance. Geometry and
            // appearance are independent; placements share both before packing.
            let key = (
                Arc::as_ptr(&placement.compiled),
                palette
                    .get(BuildingLodMaterial::Wall(
                        adventuresim_building_generator::WallMaterialClass::TimberInfill,
                    ))
                    .id(),
            );
            let index = *prototypes.entry(key).or_insert_with(|| {
                let compiled = &placement.compiled;
                let parts = [&compiled.lod0, &compiled.lod1, &compiled.lod2]
                    .into_iter()
                    .enumerate()
                    .flat_map(|(level, batches)| {
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
                    .collect();
                let index = groups.len();
                groups.push(Group {
                    parts,
                    placements: Vec::new(),
                });
                index
            });
            groups[index].placements.push(placement.transform);
        }
        groups
    }
}

pub(super) struct Group {
    pub parts: Vec<Part>,
    pub placements: Vec<Mat4>,
}

impl Group {
    pub fn from_parts(parts: impl Iterator<Item = Part>) -> Vec<Self> {
        let mut roots = HashMap::new();
        let mut groups: Vec<Self> = Vec::new();
        for part in parts {
            let index = *roots.entry(part.root).or_insert_with(|| {
                let index = groups.len();
                groups.push(Self {
                    placements: vec![part.transform],
                    parts: Vec::new(),
                });
                index
            });
            groups[index].parts.push(part);
        }
        groups
    }
}
