//! The material hair is drawn with: the standard material, shaded as
//! strands and, for cards, its atlas coverage turned into a share of each
//! pixel's samples, by `strand_shading.wgsl`.
use super::*;
use bevy::asset::embedded_asset;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

const SHADER: &str = "embedded://adventuresim_character_creator/hair_preview/strand_shading.wgsl";

/// Hair's material, for strands and cards alike.
pub(crate) type HairShading = ExtendedMaterial<StandardMaterial, StrandShading>;

/// The standard material's extension for hair: no settings of its own,
/// only its fragment shader.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub(crate) struct StrandShading {}

impl MaterialExtension for StrandShading {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

/// Draws hair with [`HairShading`].
pub(crate) struct HairShadingPlugin;

impl Plugin for HairShadingPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "strand_shading.wgsl");
        app.add_plugins(MaterialPlugin::<HairShading>::default());
    }
}
