//! Chunk-instanced rendering for dense scatter: grass, shrubs, loose stones
//! and distant tree cards.
//!
//! A batch is one mesh drawn many times. It is split into ground chunks, and
//! each chunk is an ordinary entity: Bevy's CPU frustum and range culling
//! decide which chunks a camera sees, and each visible chunk is one instanced
//! `draw_indexed` over its own instance buffer, uploaded once. Instances fade
//! in and out per instance in the shader, so crossfading distance bands hand
//! over pixel-exactly however the chunks fall. Chunks draw in the main pass
//! only, never in prepass or shadow passes.
mod draw;
mod instance;
mod material;
mod pipeline;
mod queue;
mod spawn;

use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_3d::Opaque3d,
    pbr::MeshPipelineSystems,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::ExtractComponentPlugin,
        render_asset::RenderAssetPlugin,
        render_phase::AddRenderCommand,
        render_resource::{BindGroupLayoutDescriptor, SpecializedMeshPipelines},
    },
    shader::load_shader_library,
};

pub(in crate::presentation) use instance::ChunkMaterial;
pub(crate) use instance::{ChunkInstance, FadeBand, InstanceChunk};
pub(in crate::presentation) use material::{ChunkInstancedMaterial, enable_alpha_to_coverage};
pub(in crate::presentation) use spawn::{ChunkSize, InstanceBatch, InstanceReach};

/// The renderer shared by every chunk-instanced material.
pub(in crate::presentation) struct ChunkInstancingPlugin;

/// Draws chunks whose [`ChunkMaterial`] is an `M`.
pub(in crate::presentation) struct ChunkMaterialPlugin<M>(PhantomData<M>);

/// The group-2 band layout shared by every chunk-instanced pipeline.
#[derive(Resource)]
struct ChunkBandLayout(BindGroupLayoutDescriptor);

impl Plugin for ChunkInstancingPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "chunk_instancing/chunk_instancing.wgsl");
        app.add_plugins(ExtractComponentPlugin::<InstanceChunk>::default());
        app.sub_app_mut(RenderApp)
            .insert_resource(ChunkBandLayout(draw::band_layout()))
            .add_systems(
                Render,
                draw::prepare_chunk_buffers.in_set(RenderSystems::PrepareResources),
            );
    }
}

impl<M> Default for ChunkMaterialPlugin<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<M: ChunkInstancedMaterial> Plugin for ChunkMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_asset::<M>().add_plugins((
            ExtractComponentPlugin::<ChunkMaterial<M>>::default(),
            RenderAssetPlugin::<material::PreparedChunkMaterial<M>>::default(),
        ));
        app.sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, draw::DrawChunkInstances<M>>()
            .init_resource::<SpecializedMeshPipelines<pipeline::ChunkMaterialPipeline<M>>>()
            .init_resource::<queue::QueuedChunks<M>>()
            .add_systems(
                RenderStartup,
                init_pipeline::<pipeline::ChunkMaterialPipeline<M>>.after(MeshPipelineSystems),
            )
            .add_systems(
                Render,
                queue::queue_chunks::<M>
                    .in_set(RenderSystems::QueueMeshes)
                    .after(bevy::pbr::queue_material_meshes),
            );
    }
}

fn init_pipeline<P: Resource + FromWorld>(world: &mut World) {
    let pipeline = P::from_world(world);
    world.insert_resource(pipeline);
}
