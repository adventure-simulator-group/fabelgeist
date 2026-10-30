//! Render-world chunk buffers and the draw commands that consume them.
use std::marker::PhantomData;

use bevy::{
    ecs::system::{
        SystemParamItem,
        lifetimeless::{Read, SRes},
    },
    pbr::{RenderMeshInstances, SetMeshViewBindGroup, SetMeshViewBindingArrayBindGroup},
    prelude::*,
    render::{
        mesh::{RenderMesh, RenderMeshBufferInfo, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, Buffer,
            BufferInitDescriptor, BufferUsages, PipelineCache, ShaderStages, ShaderType,
            UniformBuffer, binding_types::uniform_buffer,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

use super::{
    ChunkBandLayout, ChunkInstancedMaterial, ChunkMaterial, InstanceChunk,
    material::PreparedChunkMaterial,
};

/// Main-pass draw of one chunk.
pub(super) type DrawChunkInstances<M> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetChunkBandBindGroup<2>,
    SetChunkMaterialBindGroup<M, 3>,
    DrawChunkMesh,
);

/// A chunk's uploaded instances and band uniform.
#[derive(Component)]
pub(super) struct ChunkGpu {
    instances: Buffer,
    instance_count: u32,
    band: BindGroup,
}

/// The `ChunkBand` uniform of `chunk_instancing.wgsl`.
#[derive(ShaderType, Clone, Copy, Default)]
struct ChunkBandUniform {
    visibility_range: Vec4,
}

pub(super) struct SetChunkBandBindGroup<const I: usize>;

pub(super) struct SetChunkMaterialBindGroup<M, const I: usize>(PhantomData<M>);

/// One instanced `draw_indexed` of the chunk mesh over the chunk's instances.
pub(super) struct DrawChunkMesh;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetChunkBandBindGroup<I> {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = Read<ChunkGpu>;

    fn render<'w>(
        _item: &P,
        _view: (),
        chunk: Option<&'w ChunkGpu>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(chunk) = chunk else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &chunk.band, &[]);
        RenderCommandResult::Success
    }
}

impl<P: PhaseItem, M: ChunkInstancedMaterial, const I: usize> RenderCommand<P>
    for SetChunkMaterialBindGroup<M, I>
{
    type Param = SRes<RenderAssets<PreparedChunkMaterial<M>>>;
    type ViewQuery = ();
    type ItemQuery = Read<ChunkMaterial<M>>;

    fn render<'w>(
        _item: &P,
        _view: (),
        material: Option<&'w ChunkMaterial<M>>,
        materials: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(prepared) = material.and_then(|material| materials.into_inner().get(&material.0))
        else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &prepared.bind_group, &[]);
        RenderCommandResult::Success
    }
}

impl<P: PhaseItem> RenderCommand<P> for DrawChunkMesh {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<ChunkGpu>;

    fn render<'w>(
        item: &P,
        _view: (),
        chunk: Option<&'w ChunkGpu>,
        (meshes, mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();
        let Some(chunk) = chunk else {
            return RenderCommandResult::Skip;
        };
        let Some(mesh_id) = mesh_instances
            .render_mesh_queue_data(item.main_entity())
            .map(|instance| instance.mesh_asset_id())
        else {
            return RenderCommandResult::Skip;
        };
        let (Some(mesh), Some(vertices), Some(indices)) = (
            meshes.into_inner().get(mesh_id),
            mesh_allocator.mesh_vertex_slice(&mesh_id),
            mesh_allocator.mesh_index_slice(&mesh_id),
        ) else {
            return RenderCommandResult::Skip;
        };
        let RenderMeshBufferInfo::Indexed {
            index_format,
            count,
        } = &mesh.buffer_info
        else {
            return RenderCommandResult::Skip;
        };
        pass.set_vertex_buffer(0, vertices.buffer.slice(..));
        pass.set_vertex_buffer(1, chunk.instances.slice(..));
        pass.set_index_buffer(indices.buffer.slice(..), *index_format);
        pass.draw_indexed(
            indices.range.start..indices.range.start + count,
            vertices.range.start as i32,
            0..chunk.instance_count,
        );
        RenderCommandResult::Success
    }
}

/// Uploads a chunk's instances and band once, and again whenever the
/// extracted chunk changes.
#[expect(
    clippy::type_complexity,
    reason = "the Bevy query selects new and changed chunks"
)]
pub(super) fn prepare_chunk_buffers(
    mut commands: Commands,
    chunks: Query<(Entity, &InstanceChunk), Or<(Changed<InstanceChunk>, Without<ChunkGpu>)>>,
    band_layout: Res<ChunkBandLayout>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let layout = pipeline_cache.get_bind_group_layout(&band_layout.0);
    for (entity, chunk) in &chunks {
        let instance_count = u32::try_from(chunk.instances.len()).unwrap_or(u32::MAX);
        if instance_count == 0 {
            continue;
        }
        let instances = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("chunk_instances"),
            contents: bytemuck::cast_slice(&chunk.instances),
            usage: BufferUsages::VERTEX,
        });
        let mut band = UniformBuffer::from(ChunkBandUniform {
            visibility_range: chunk.band.shader_range(),
        });
        band.write_buffer(&render_device, &render_queue);
        let Some(band_binding) = band.binding() else {
            continue;
        };
        let band = render_device.create_bind_group(
            "chunk_band_bind_group",
            &layout,
            &BindGroupEntries::single(band_binding),
        );
        commands.entity(entity).insert(ChunkGpu {
            instances,
            instance_count,
            band,
        });
    }
}

/// The group-2 layout every chunk-instanced pipeline binds its band with.
pub(super) fn band_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "chunk_band_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::VERTEX_FRAGMENT,
            uniform_buffer::<ChunkBandUniform>(false),
        ),
    )
}
