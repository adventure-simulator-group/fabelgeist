//! Explicit ordinary vertex layouts and analytic leaf cutouts in every pass.

use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, FragmentState, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
        storage::ShaderBuffer,
    },
    shader::ShaderRef,
};

use crate::prepared::GrownVertexEncoding;
use crate::render::address::VERTEX_ADDRESS;

const PREPASS_DEFINE: &str = "PREPASS_PIPELINE";
const LEAF_DEFINE: &str = "GROWN_LEAF";
const ADDRESS_DEFINE: &str = "GROWN_ADDRESS";
const EXPANDED_DEFINE: &str = "GROWN_EXPANDED";

const SHADER: &str = "embedded://adventuresim_environment/render/grown/shaders/material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, PartialEq)]
#[bind_group_data(GrownVertexEncoding)]
pub struct GrownWoodMaterial {
    #[uniform(0)]
    pub(super) shape: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    pub(super) records: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    pub(super) species: Handle<ShaderBuffer>,
    #[storage(3, read_only, visibility(vertex, fragment))]
    pub(super) plants: Handle<ShaderBuffer>,
    pub(super) encoding: GrownVertexEncoding,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, PartialEq)]
#[bind_group_data(GrownVertexEncoding)]
pub struct GrownLeafMaterial {
    #[uniform(0)]
    pub(super) shape: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    pub(super) records: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    pub(super) species: Handle<ShaderBuffer>,
    pub(super) encoding: GrownVertexEncoding,
}

impl From<&GrownWoodMaterial> for GrownVertexEncoding {
    fn from(material: &GrownWoodMaterial) -> Self {
        material.encoding
    }
}

impl From<&GrownLeafMaterial> for GrownVertexEncoding {
    fn from(material: &GrownLeafMaterial) -> Self {
        material.encoding
    }
}

impl Material for GrownWoodMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
    fn prepass_vertex_shader() -> ShaderRef {
        SHADER.into()
    }
    fn prepass_fragment_shader() -> ShaderRef {
        SHADER.into()
    }
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize(descriptor, layout, key.bind_group_data)?;
        Ok(())
    }
}

impl Material for GrownLeafMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
    fn prepass_vertex_shader() -> ShaderRef {
        SHADER.into()
    }
    fn prepass_fragment_shader() -> ShaderRef {
        SHADER.into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::AlphaToCoverage
    }
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let prepass = descriptor
            .vertex
            .shader_defs
            .contains(&PREPASS_DEFINE.into());
        if prepass && descriptor.fragment.is_none() {
            descriptor.fragment = Some(FragmentState {
                shader: descriptor.vertex.shader.clone(),
                shader_defs: descriptor.vertex.shader_defs.clone(),
                targets: Vec::new(),
                ..default()
            });
        }
        specialize(descriptor, layout, key.bind_group_data)?;
        descriptor.vertex.shader_defs.push(LEAF_DEFINE.into());
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.shader_defs.push(LEAF_DEFINE.into());
        }
        descriptor.primitive.cull_mode = None;
        descriptor.multisample.alpha_to_coverage_enabled =
            descriptor.multisample.count > 1 && !prepass;
        Ok(())
    }
}

fn specialize(
    descriptor: &mut RenderPipelineDescriptor,
    layout: &MeshVertexBufferLayoutRef,
    encoding: GrownVertexEncoding,
) -> Result<(), SpecializedMeshPipelineError> {
    descriptor.vertex.buffers = vec![layout.0.get_layout(&[
        Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
        Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
        Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
        VERTEX_ADDRESS.at_shader_location(3),
    ])?];
    let definition = match encoding {
        GrownVertexEncoding::Address => ADDRESS_DEFINE,
        GrownVertexEncoding::Expanded => EXPANDED_DEFINE,
    };
    descriptor.vertex.shader_defs.push(definition.into());
    if let Some(fragment) = &mut descriptor.fragment {
        fragment.shader_defs.push(definition.into());
    }
    Ok(())
}
