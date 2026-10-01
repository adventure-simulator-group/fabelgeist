//! Compile each distinct component once across every generated city building.
use super::*;
use adventuresim_building_generator::{BuildingKit, TimberComponent};

#[derive(Default)]
pub(super) struct ComponentCache {
    meshes: std::collections::HashMap<([u32; 3], bool), Vec<CompiledBuildingBatch>>,
}

impl ComponentCache {
    fn get(
        &mut self,
        component: TimberComponent,
        meshes: &mut Assets<Mesh>,
    ) -> &[CompiledBuildingBatch] {
        self.meshes
            .entry((
                component.size_metres.to_array().map(f32::to_bits),
                component.interior,
            ))
            .or_insert_with(|| {
                component
                    .meshes()
                    .iter()
                    .map(|batch| {
                        let mut mesh = recipe_mesh(batch, Vec3::ZERO);
                        mesh.asset_usage = RenderAssetUsages::MAIN_WORLD;
                        CompiledBuildingBatch {
                            material: batch.material,
                            mesh: meshes.add(mesh),
                            triangles: batch.indices.len() / 3,
                            transform: Mat4::IDENTITY,
                            uv_offset: Vec2::ZERO,
                        }
                    })
                    .collect()
            })
    }

    pub(super) fn append(
        &mut self,
        kit: &BuildingKit<'_>,
        compiled: &mut CompiledBuildingLevels,
        meshes: &mut Assets<Mesh>,
    ) {
        for instance in &kit.instances {
            if compiled.detail == BuildingDetail::Facade && !instance.facade {
                continue;
            }
            for batch in self.get(instance.component, meshes) {
                let mut batch = batch.clone();
                batch.transform =
                    Mat4::from_translation(-compiled.local_origin) * instance.transform;
                batch.uv_offset = instance.uv_offset;
                if compiled.detail == BuildingDetail::Static {
                    compiled.lod0.push(batch.clone());
                }
                if instance.facade {
                    compiled.lod1.push(batch);
                }
            }
        }
    }
}
