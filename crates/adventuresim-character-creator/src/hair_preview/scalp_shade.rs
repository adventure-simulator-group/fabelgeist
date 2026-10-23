//! The scalp under the hair, painted with the hair lying on it and shaded
//! as the hair is.
//!
//! Between strands and cards the skin would show bare. The scalp cap is
//! drawn as hair instead, from the textures baked from the groom: the
//! strands lying along the scalp, their shade and the way each runs, lit by
//! the hair's own material and strand normals ([`ScalpShading`]), so the
//! hair painted on the scalp and the hair drawn over it look alike. Where
//! the scalp is bare -- at a parting -- the textures turn clear and the skin
//! beneath shows. Only the cap is painted -- the body's own skin, the neck
//! and the shoulders hair falls over, keeps its colour -- and the cap comes
//! and goes with the hair it lies under. The cap is blended over the skin,
//! fading out towards its edge.
use super::*;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub(super) const SHADER: &str =
    "embedded://adventuresim_character_creator/hair_preview/scalp_shading.wgsl";

/// The scalp's material: the hair's, shaded by `scalp_shading.wgsl`.
pub(crate) type ScalpShading = ExtendedMaterial<StandardMaterial, ScalpStrands>;

/// The standard material's extension for the scalp: the baked strands
/// texture, the way each strand runs across it.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub(crate) struct ScalpStrands {
    #[texture(100)]
    #[sampler(101)]
    strands: Handle<Image>,
}

impl MaterialExtension for ScalpStrands {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

/// The scalp's baked textures, as images.
#[derive(Clone)]
pub(super) struct ScalpImages {
    /// Shade as grey and coverage as alpha: the base colour's texture.
    pub(super) shade: Handle<Image>,
    /// The way each strand runs, and its shade.
    pub(super) strands: Handle<Image>,
}

/// The material the scalp cap under hair of `design` is drawn with: the
/// hair's, its colour shaded by the baked shade, blended over the skin.
pub(super) fn material(design: HairDesign, images: &ScalpImages) -> ScalpShading {
    ScalpShading {
        base: StandardMaterial {
            base_color_texture: Some(images.shade.clone()),
            alpha_mode: AlphaMode::Blend,
            ..assets::hair_surface(design)
        },
        extension: ScalpStrands {
            strands: images.strands.clone(),
        },
    }
}

/// The cap's vertex colours: white, as the material carries the hair's
/// colour, showing by `opacity`.
pub(super) fn colors(opacity: &[f32]) -> Vec<[f32; 4]> {
    opacity
        .iter()
        .map(|&opacity| [1.0, 1.0, 1.0, opacity])
        .collect()
}
