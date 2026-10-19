//! The shared geometry shaders add one placement buffer to the record bindings.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::{
    render_resource::{AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError},
    storage::ShaderBuffer,
};
use bevy::shader::ShaderRef;
use bevy_eidolon::prelude::*;

use super::super::materials::{BRANCH_SHADER, CULL_SHADER, LEAF_SHADER};
const SHARED_DEFINE: &str = "FOREST_SHARED_TREES";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TreeBranchMaterial {
    #[uniform(0)]
    pub(super) lod: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    pub(super) branches: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    pub(super) species: Handle<ShaderBuffer>,
    #[storage(3, read_only, visibility(vertex, fragment))]
    pub(super) plants: Handle<ShaderBuffer>,
    #[storage(4, read_only, visibility(vertex))]
    pub(super) placements: Handle<ShaderBuffer>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TreeLeafMaterial {
    #[uniform(0)]
    pub(super) lod: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    pub(super) leaves: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    pub(super) species: Handle<ShaderBuffer>,
    #[storage(4, read_only, visibility(vertex))]
    pub(super) placements: Handle<ShaderBuffer>,
}

impl InstancedMaterial for TreeBranchMaterial {
    fn cull_shader() -> ShaderRef {
        CULL_SHADER.into()
    }
    fn vertex_shader() -> ShaderRef {
        BRANCH_SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        BRANCH_SHADER.into()
    }
    fn disable_prepass(&self) -> bool {
        true
    }
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize_shared(descriptor, layout)?;
        Ok(())
    }
}

impl InstancedMaterial for TreeLeafMaterial {
    fn cull_shader() -> ShaderRef {
        CULL_SHADER.into()
    }
    fn vertex_shader() -> ShaderRef {
        LEAF_SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        LEAF_SHADER.into()
    }
    fn disable_prepass(&self) -> bool {
        true
    }
    fn double_sided(&self) -> bool {
        true
    }
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize_shared(descriptor, layout)?;
        descriptor.primitive.cull_mode = None;
        descriptor.multisample.alpha_to_coverage_enabled = descriptor.multisample.count > 1;
        Ok(())
    }
}

fn specialize_shared(
    descriptor: &mut RenderPipelineDescriptor,
    layout: &MeshVertexBufferLayoutRef,
) -> Result<(), SpecializedMeshPipelineError> {
    descriptor.vertex.buffers = vec![
        layout
            .0
            .get_layout(&[super::VERTEX_ADDRESS.at_shader_location(0)])?,
    ];
    descriptor.vertex.shader_defs.push(SHARED_DEFINE.into());
    if let Some(fragment) = &mut descriptor.fragment {
        fragment.shader_defs.push(SHARED_DEFINE.into());
    }
    Ok(())
}
