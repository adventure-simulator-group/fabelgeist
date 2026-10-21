//! Actual same-frame renderer mesh allocation, material and pipeline joins.
use super::*;
use availability::{MissingBinding, MissingMemberResource};
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

pub(super) use availability::Availability;

mod availability;

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

struct BindingSources<'a> {
    meshes: &'a RenderAssets<RenderMesh>,
    instances: &'a RenderMeshInstances,
    materials: &'a RenderMaterialInstances,
    prepared: &'a ErasedRenderAssets<PreparedMaterial>,
    allocator: &'a MeshAllocator,
    specializations: &'a SpecializedMaterialPipelineCache,
    pipelines: &'a PipelineCache,
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
    ) -> Result<Availability<Vec<Self>>> {
        if specializations.get(&pose.camera).is_none() {
            return Ok(Availability::Missing(MissingBinding::ViewSpecializations));
        }
        let sources = BindingSources {
            meshes,
            instances,
            materials,
            prepared,
            allocator,
            specializations,
            pipelines,
        };
        let mut members = Vec::with_capacity(pose.expected.len());
        for member in &pose.expected {
            // Native visibility only gates availability of renderer records.
            // The full pose membership remains required by GPU validation.
            if member.native_visibility == NativeVisibility::NotVisible {
                continue;
            }
            match sources.resolve(pose.camera, member)? {
                Availability::Ready(member) => members.push(member),
                Availability::Missing(reason) => return Ok(Availability::Missing(reason)),
            }
        }
        Ok(Availability::Ready(members))
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

impl BindingSources<'_> {
    fn resolve(
        &self,
        camera: RetainedViewEntity,
        member: &TaggedMember,
    ) -> Result<Availability<RenderMember>> {
        let unavailable = |resource| {
            Availability::Missing(MissingBinding::Member {
                member: Box::new(member.clone()),
                resource,
            })
        };
        let main_entity: MainEntity = member.entity.into();
        let extraction = match self.instances.mesh_asset_id(main_entity) {
            Some(extracted) => {
                ensure!(
                    extracted == member.mesh,
                    "actual extracted mesh differs from held member"
                );
                Extraction::Present
            }
            None => Extraction::Absent,
        };
        let Some(material) = self.materials.instances.get(&main_entity) else {
            return Ok(unavailable(MissingMemberResource::MaterialInstance));
        };
        ensure!(
            material.asset_id == member.material,
            "actual extracted material differs from held member"
        );
        let Some(material) = self.prepared.get(member.material) else {
            return Ok(unavailable(MissingMemberResource::PreparedMaterial));
        };
        let Some(mesh) = self.meshes.get(member.mesh) else {
            return Ok(unavailable(MissingMemberResource::Mesh));
        };
        let Some(binding) = MeshBinding::read(member.mesh, mesh, self.allocator) else {
            return Ok(unavailable(MissingMemberResource::MeshAllocation {
                vertex_slice_present: self.allocator.mesh_vertex_slice(&member.mesh).is_some(),
                slabs_present: self.allocator.mesh_slabs(&member.mesh).is_some(),
                indexed: matches!(mesh.buffer_info, RenderMeshBufferInfo::Indexed { .. }),
                index_slice_present: self.allocator.mesh_index_slice(&member.mesh).is_some(),
            }));
        };
        let Some(pipeline) = self
            .specializations
            .get(&camera)
            .and_then(|view| view.get(&main_entity))
        else {
            return Ok(unavailable(MissingMemberResource::EntitySpecialization {
                mesh_queue_data_present: self
                    .instances
                    .render_mesh_queue_data(main_entity)
                    .is_some(),
            }));
        };
        if self.pipelines.get_render_pipeline(*pipeline).is_none() {
            return Ok(unavailable(MissingMemberResource::CompiledPipeline {
                pipeline: format!("{pipeline:?}"),
                preparation: self.pipelines.get_render_pipeline_state(*pipeline).into(),
            }));
        }
        Ok(Availability::Ready(RenderMember {
            member: member.clone(),
            extraction,
            pipeline: *pipeline,
            binding,
            material_group: material.binding.group.0,
            material_slot: material.binding.slot.0,
        }))
    }
}

pub(super) fn material_type(id: UntypedAssetId) -> std::any::TypeId {
    id.type_id()
}
