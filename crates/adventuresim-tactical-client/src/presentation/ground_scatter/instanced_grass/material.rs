//! The instanced grass material: the uniforms the sward shader reads and the
//! pipeline tweaks thin double-sided ribbons need.

use bevy::{
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use bevy_eidolon::prelude::InstancedMaterial;

const GRASS_INSTANCED_SHADER: &str = "shaders/tactical_grass_instanced.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, TacticalGrassInstancedUniform)]
pub(in crate::presentation) struct TacticalGrassInstancedMaterial {
    /// Wind direction xy, strength, time scale.
    pub wind: Vec4,
    /// Interactor position (xyz) and interaction radius (w; 0 disables).
    pub interaction: Vec4,
    /// Interactor smoothed velocity (xyz) and push strength.
    pub interaction_motion: Vec4,
    /// Root occlusion, dryness lane, authored lean, width compensation.
    pub params: Vec4,
    /// y scales the flat ambient term to approximate the skipped image-based
    /// lighting. Every tier now shades fast (ambient + wrapped Lambert + one
    /// clamped shadow fetch), so x/z/w are reserved.
    pub shading: Vec4,
}

#[derive(Clone, Default, ShaderType)]
pub(in crate::presentation) struct TacticalGrassInstancedUniform {
    wind: Vec4,
    interaction: Vec4,
    interaction_motion: Vec4,
    params: Vec4,
    shading: Vec4,
}

impl From<&TacticalGrassInstancedMaterial> for TacticalGrassInstancedUniform {
    fn from(material: &TacticalGrassInstancedMaterial) -> Self {
        Self {
            wind: material.wind,
            interaction: material.interaction,
            interaction_motion: material.interaction_motion,
            params: material.params,
            shading: material.shading,
        }
    }
}

impl InstancedMaterial for TacticalGrassInstancedMaterial {
    fn vertex_shader() -> ShaderRef {
        GRASS_INSTANCED_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        GRASS_INSTANCED_SHADER.into()
    }

    /// Legacy grass renders without a prepass; keep the instanced sward
    /// identical so the depth interplay with the terrain detail patch and
    /// transparent weather stays unchanged.
    fn disable_prepass(&self) -> bool {
        true
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Thin ribbons are shaded double-sided, and the LOD dither converts
        // to hardware coverage samples under MSAA exactly like the legacy
        // AlphaToCoverage foliage material.
        descriptor.primitive.cull_mode = None;
        if descriptor.multisample.count > 1 {
            descriptor.multisample.alpha_to_coverage_enabled = true;
        }
        Ok(())
    }
}
