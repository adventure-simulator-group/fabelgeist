//! Actual same-frame renderer mesh allocation, material and pipeline joins.
use super::*;
use bevy::{
    asset::UntypedAssetId,
    pbr::{
        PreparedMaterial, RenderMaterialInstances, RenderMeshInstances,
        SpecializedMaterialPipelineCache,
    },
    render::{
        erased_render_asset::ErasedRenderAssets,
        mesh::{
            RenderMesh, RenderMeshBufferInfo,
            allocator::{MeshAllocator, MeshSlabs},
        },
        render_asset::RenderAssets,
        render_resource::{BufferId, CachedRenderPipelineId, IndexFormat, PipelineCache},
        sync_world::MainEntity,
    },
};
use members::NativeVisibility;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeshBinding {
    pub slabs: MeshSlabs,
    pub vertex_buffer: BufferId,
    pub index_buffer: Option<BufferId>,
    pub geometry: Geometry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Geometry {
    Indexed {
        format: IndexFormat,
        count: u32,
        first_index: u32,
        first_vertex: u32,
    },
    NonIndexed {
        count: u32,
        first_vertex: u32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RenderMember {
    pub member: TaggedMember,
    pub extraction: Extraction,
    pub pipeline: CachedRenderPipelineId,
    pub binding: MeshBinding,
    pub material_group: u32,
    pub material_slot: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Extraction {
    Present,
    Absent,
}

impl MeshBinding {
    pub(super) fn read(
        id: AssetId<Mesh>,
        mesh: &RenderMesh,
        allocator: &MeshAllocator,
    ) -> Option<Self> {
        let vertex = allocator.mesh_vertex_slice(&id)?;
        let slabs = allocator.mesh_slabs(&id)?;
        let (geometry, index_buffer) = match &mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                count,
                index_format,
            } => {
                let indices = allocator.mesh_index_slice(&id)?;
                (
                    Geometry::Indexed {
                        format: *index_format,
                        count: *count,
                        first_index: indices.range.start,
                        first_vertex: vertex.range.start,
                    },
                    Some(indices.buffer.id()),
                )
            }
            RenderMeshBufferInfo::NonIndexed => (
                Geometry::NonIndexed {
                    count: mesh.vertex_count,
                    first_vertex: vertex.range.start,
                },
                None,
            ),
        };
        Some(Self {
            slabs,
            vertex_buffer: vertex.buffer.id(),
            index_buffer,
            geometry,
        })
    }

    pub(super) fn report(self) -> serde_json::Value {
        serde_json::json!({"slabs":format!("{:?}",self.slabs),
            "vertex_buffer":format!("{:?}",self.vertex_buffer),
            "index_buffer":self.index_buffer.map(|id|format!("{id:?}")),
            "geometry":self.geometry.report()})
    }
}

impl Geometry {
    pub(super) fn indexed(self) -> bool {
        matches!(self, Self::Indexed { .. })
    }

    pub(super) fn first_vertex(self) -> u32 {
        match self {
            Self::Indexed { first_vertex, .. } | Self::NonIndexed { first_vertex, .. } => {
                first_vertex
            }
        }
    }

    pub(super) fn report(self) -> serde_json::Value {
        match self {
            Self::Indexed {
                format,
                count,
                first_index,
                first_vertex,
            } => serde_json::json!({
                "kind":"indexed", "format":format!("{format:?}"), "count":count,
                "first_index":first_index, "first_vertex":first_vertex}),
            Self::NonIndexed {
                count,
                first_vertex,
            } => serde_json::json!({
                "kind":"non_indexed", "count":count, "first_vertex":first_vertex}),
        }
    }
}

impl RenderMember {
    #[expect(
        clippy::too_many_arguments,
        reason = "one native renderer snapshot joins its existing asset, material and specialization resources"
    )]
    pub(super) fn collect(
        pose: &Pose,
        meshes: &RenderAssets<RenderMesh>,
        instances: &RenderMeshInstances,
        materials: &RenderMaterialInstances,
        prepared: &ErasedRenderAssets<PreparedMaterial>,
        allocator: &MeshAllocator,
        specializations: &SpecializedMaterialPipelineCache,
        pipelines: &PipelineCache,
    ) -> Result<Option<Vec<Self>>> {
        let Some(view_pipelines) = specializations.get(&pose.camera) else {
            return Ok(None);
        };
        let mut members = Vec::with_capacity(pose.expected.len());
        for member in &pose.expected {
            // Native visibility only gates availability of renderer records.
            // The full pose membership remains required by GPU validation.
            if member.native_visibility == NativeVisibility::NotVisible {
                continue;
            }
            let main_entity: MainEntity = member.entity.into();
            let extraction = match instances.mesh_asset_id(main_entity) {
                Some(extracted) => {
                    ensure!(
                        extracted == member.mesh,
                        "actual extracted mesh differs from held member"
                    );
                    Extraction::Present
                }
                None => Extraction::Absent,
            };
            let Some(material) = materials.instances.get(&main_entity) else {
                return Ok(None);
            };
            ensure!(
                material.asset_id == member.material,
                "actual extracted material differs from held member"
            );
            let Some(material) = prepared.get(member.material) else {
                return Ok(None);
            };
            let Some(mesh) = meshes.get(member.mesh) else {
                return Ok(None);
            };
            let Some(binding) = MeshBinding::read(member.mesh, mesh, allocator) else {
                return Ok(None);
            };
            let Some(pipeline) = view_pipelines.get(&main_entity) else {
                return Ok(None);
            };
            if pipelines.get_render_pipeline(*pipeline).is_none() {
                return Ok(None);
            }
            members.push(Self {
                member: member.clone(),
                extraction,
                pipeline: *pipeline,
                binding,
                material_group: material.binding.group.0,
                material_slot: material.binding.slot.0,
            });
        }
        Ok(Some(members))
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"member":self.member.report(), "extraction":self.extraction,
            "pipeline":format!("{:?}",self.pipeline),
            "binding":self.binding.report(), "material_group":self.material_group,
            "material_slot":self.material_slot})
    }

    pub(super) fn material_type(&self) -> std::any::TypeId {
        self.member.material.type_id()
    }
}

pub(super) fn material_type(id: UntypedAssetId) -> std::any::TypeId {
    id.type_id()
}
