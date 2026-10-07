//! The character's hair as a rigged shell: cards skinned wholly to the head,
//! their atlas tinted with the hair's colour and cut out by its coverage.
use super::*;
use crate::character_morphs::WholeJoint;
use crate::hair_preview;
use adventuresim_character_creator::export::SurfaceTextures;
use adventuresim_character_creator::hair::{GroomedHair, HAIR_ROUGHNESS, HairCards, HairDesign};
use image::ImageEncoder;

/// The shell's name in the exported scene.
const SHELL_NAME: &str = "hair";
/// A tangent-space normal map that leaves every normal as it is.
const FLAT_NORMAL: [u8; 4] = [128, 128, 255, 255];

/// Hair grown and placed for export, owning what its shell borrows.
pub(super) struct ExportedHair {
    cards: HairCards,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
    textures: SurfaceTextures,
    morphs: Vec<MorphDelta>,
}

impl ExportedHair {
    /// Grow `design`, place it on the `generated` body's head, and follow
    /// the head through every one of the body's morph channels.
    pub(super) fn grow(
        model: &BodyModel,
        generated: &GeneratedCharacter,
        morphs: &CharacterMorphs,
        design: HairDesign,
    ) -> Result<Self> {
        let hair = GroomedHair::grow(design.preset);
        let (cards, placement) = hair_preview::place(&hair, model, generated)?;
        let count = cards.positions.len();
        let skin = WholeJoint::new(placement.head_joint);
        Ok(Self {
            morphs: morphs.hair(model, &hair, &cards)?,
            textures: SurfaceTextures {
                base_color_png: Some(tinted_atlas(&hair, design)?),
                normal_png: encode(&FLAT_NORMAL, 1, 1)?,
                occlusion_png: None,
                cutout: true,
            },
            cards,
            joint_indices: vec![skin.indices; count],
            joint_weights: vec![skin.weights; count],
        })
    }

    pub(super) fn targets(&self) -> Vec<RiggedMorphTarget<'_>> {
        self.morphs.iter().map(MorphDelta::rigged).collect()
    }

    pub(super) fn shell<'a>(&'a self, targets: &'a [RiggedMorphTarget<'a>]) -> RiggedShell<'a> {
        RiggedShell {
            plate_edges: &[],
            surface: None,
            textures: Some(self.textures.clone()),
            texcoords: Some(&self.cards.texcoords),
            hinge: None,
            name: SHELL_NAME,
            positions: &self.cards.positions,
            normals: &self.cards.normals,
            faces: &self.cards.faces,
            joint_indices: Some(&self.joint_indices),
            joint_weights: Some(&self.joint_weights),
            morph_targets: targets,
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: HAIR_ROUGHNESS,
        }
    }
}

/// The atlas's largest level in the hair's colour, sRGB, its coverage as
/// alpha: glTF takes a texture's colour as the whole base colour.
fn tinted_atlas(hair: &GroomedHair, design: HairDesign) -> Result<Vec<u8>> {
    let levels = hair.atlas_levels();
    let level = levels.first().context("the hair atlas has no levels")?;
    let color = design.pigment.color();
    let tinted: Vec<u8> = level
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[shade, _, _, coverage]| {
            let shade = f32::from(shade) / f32::from(u8::MAX);
            let [red, green, blue] = color.map(|channel| srgb_byte(channel * shade));
            [red, green, blue, coverage]
        })
        .collect();
    encode(&tinted, level.width, level.height)
}

/// A linear channel as an sRGB byte.
fn srgb_byte(linear: f32) -> u8 {
    let encoded = Color::linear_rgb(linear, 0.0, 0.0).to_srgba().red;
    (encoded.clamp(0.0, 1.0) * f32::from(u8::MAX)).round() as u8
}

fn encode(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes).write_image(
        pixels,
        width,
        height,
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(bytes)
}
