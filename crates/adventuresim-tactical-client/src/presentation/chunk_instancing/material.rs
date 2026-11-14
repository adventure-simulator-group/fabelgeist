//! The material contract of chunk-instanced batches and its render-world
//! preparation into a bind group.
use bevy::{
    asset::AssetId,
    core_pipeline::core_3d::Opaque3d,
    ecs::system::{SystemParamItem, lifetimeless::SRes},
    mesh::MeshVertexBufferLayoutRef,
    prelude::*,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_phase::{DrawFunctionId, DrawFunctions},
        render_resource::{
            AsBindGroup, AsBindGroupError, BindGroup, BindGroupEntry, PipelineCache,
            RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
        renderer::RenderDevice,
    },
    shader::ShaderRef,
};

use super::{draw::DrawChunkInstances, pipeline::ChunkMaterialPipeline};

/// A material drawn by the chunk-instanced renderer. Its shader imports
/// `fablegeist::chunk_instancing`, binds the material at group 3, and
/// provides `vertex` and `fragment` entry points.
pub(in crate::presentation) trait ChunkInstancedMaterial:
    Asset + AsBindGroup<Data: Clone + Eq + std::hash::Hash> + Clone + Sized
{
    fn shader() -> ShaderRef;

    /// Adjusts the material's main-pass pipeline.
    fn specialize(
        _descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        Ok(())
    }
}

/// A material's bind group and the draw function that uses it.
pub(super) struct PreparedChunkMaterial<M: ChunkInstancedMaterial> {
    pub bind_group: BindGroup,
    pub key: M::Data,
    pub draw_main: DrawFunctionId,
}

impl<M: ChunkInstancedMaterial> RenderAsset for PreparedChunkMaterial<M> {
    type SourceAsset = M;
    type Param = (
        SRes<RenderDevice>,
        SRes<ChunkMaterialPipeline<M>>,
        SRes<PipelineCache>,
        <M as AsBindGroup>::Param,
        SRes<DrawFunctions<Opaque3d>>,
    );

    fn prepare_asset(
        material: Self::SourceAsset,
        _asset_id: AssetId<Self::SourceAsset>,
        (render_device, pipeline, pipeline_cache, material_param, draw_main): &mut SystemParamItem<
            Self::Param,
        >,
        _previous_asset: Option<&Self>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        let layout = pipeline_cache.get_bind_group_layout(&pipeline.material_layout);
        let unprepared =
            match material.unprepared_bind_group(&layout, render_device, material_param, false) {
                Ok(unprepared) => unprepared,
                Err(AsBindGroupError::RetryNextUpdate) => {
                    return Err(PrepareAssetError::RetryNextUpdate(material));
                }
                Err(error) => return Err(PrepareAssetError::AsBindGroupError(error)),
            };
        let entries = unprepared
            .bindings
            .iter()
            .map(|(binding, resource)| BindGroupEntry {
                binding: *binding,
                resource: resource.get_binding(),
            })
            .collect::<Vec<_>>();
        let Some(draw_main) = draw_main.read().get_id::<DrawChunkInstances<M>>() else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };
        Ok(Self {
            bind_group: render_device.create_bind_group(
                Some("chunk_instanced_material_bind_group"),
                &layout,
                &entries,
            ),
            key: material.bind_group_data(),
            draw_main,
        })
    }
}

/// Instanced cutouts resolve fractional alpha to MSAA sample coverage.
pub(in crate::presentation) fn enable_alpha_to_coverage(descriptor: &mut RenderPipelineDescriptor) {
    if descriptor.multisample.count > 1 {
        descriptor.multisample.alpha_to_coverage_enabled = true;
    }
}
