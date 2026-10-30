//! Main-pass pipelines of chunk-instanced materials.
//!
//! Each starts from Bevy's mesh pipeline, replaces bind group 2 with the
//! chunk band and bind group 3 with the material, and appends the
//! per-instance vertex stream. Chunks never cast shadows.
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bevy::{
    mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout},
    pbr::{MeshPipeline, MeshPipelineKey},
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayoutDescriptor, RenderPipelineDescriptor, SpecializedMeshPipeline,
            SpecializedMeshPipelineError, VertexAttribute, VertexFormat, VertexStepMode,
        },
        renderer::RenderDevice,
    },
};

use super::{ChunkBandLayout, ChunkInstance, ChunkInstancedMaterial};

/// The pipeline key: the view and mesh key plus the material's own data.
pub(super) struct ChunkPipelineKey<M: ChunkInstancedMaterial> {
    pub mesh_key: MeshPipelineKey,
    pub material_key: M::Data,
}

/// Main-pass pipeline inputs of one material.
#[derive(Resource)]
pub(super) struct ChunkMaterialPipeline<M: ChunkInstancedMaterial> {
    mesh_pipeline: MeshPipeline,
    band_layout: BindGroupLayoutDescriptor,
    pub material_layout: BindGroupLayoutDescriptor,
    shader: Handle<Shader>,
    material: PhantomData<M>,
}

impl<M: ChunkInstancedMaterial> Clone for ChunkPipelineKey<M> {
    fn clone(&self) -> Self {
        Self {
            mesh_key: self.mesh_key,
            material_key: self.material_key.clone(),
        }
    }
}

impl<M: ChunkInstancedMaterial> PartialEq for ChunkPipelineKey<M> {
    fn eq(&self, other: &Self) -> bool {
        self.mesh_key == other.mesh_key && self.material_key == other.material_key
    }
}

impl<M: ChunkInstancedMaterial> Eq for ChunkPipelineKey<M> {}

impl<M: ChunkInstancedMaterial> Hash for ChunkPipelineKey<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.mesh_key.hash(state);
        self.material_key.hash(state);
    }
}

impl<M: ChunkInstancedMaterial> SpecializedMeshPipeline for ChunkMaterialPipeline<M> {
    type Key = ChunkPipelineKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh_key, layout)?;
        // Groups 0 and 1 stay Bevy's view layouts; the per-entity mesh
        // uniforms at group 2 give way to the chunk band.
        descriptor.layout.truncate(2);
        descriptor
            .layout
            .extend([self.band_layout.clone(), self.material_layout.clone()]);
        descriptor
            .vertex
            .shader_defs
            .push("VISIBILITY_RANGE_DITHER".into());
        descriptor.vertex.shader = self.shader.clone();
        if let Some(fragment) = descriptor.fragment.as_mut() {
            if let Some(Some(target)) = fragment.targets.first_mut() {
                target.blend = None;
            }
            fragment.shader_defs.push("VISIBILITY_RANGE_DITHER".into());
            fragment.shader = self.shader.clone();
        }
        M::specialize(&mut descriptor, layout, key.material_key)?;
        descriptor.vertex.buffers.push(instance_buffer_layout());
        Ok(descriptor)
    }
}

impl<M: ChunkInstancedMaterial> FromWorld for ChunkMaterialPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let material_layout = BindGroupLayoutDescriptor::new(
            format!(
                "chunk_instanced_material_layout_{}",
                std::any::type_name::<M>()
            ),
            &M::bind_group_layout_entries(render_device, false),
        );
        Self {
            mesh_pipeline: world.resource::<MeshPipeline>().clone(),
            band_layout: world.resource::<ChunkBandLayout>().0.clone(),
            material_layout,
            shader: material_shader::<M>(world),
            material: PhantomData,
        }
    }
}

fn material_shader<M: ChunkInstancedMaterial>(world: &World) -> Handle<Shader> {
    match M::shader() {
        bevy::shader::ShaderRef::Handle(handle) => handle,
        bevy::shader::ShaderRef::Path(path) => world.resource::<AssetServer>().load(path),
        bevy::shader::ShaderRef::Default => Handle::default(),
    }
}

/// The `@location(8..10)` instance stream of `chunk_instancing.wgsl`.
fn instance_buffer_layout() -> VertexBufferLayout {
    let rotation_offset = VertexFormat::Float32x4.size();
    let seed_offset = rotation_offset + VertexFormat::Float32.size();
    VertexBufferLayout {
        array_stride: size_of::<ChunkInstance>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: vec![
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 8,
            },
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: rotation_offset,
                shader_location: 9,
            },
            VertexAttribute {
                format: VertexFormat::Uint32,
                offset: seed_offset,
                shader_location: 10,
            },
        ],
    }
}
