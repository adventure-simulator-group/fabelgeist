//! The studio's body and morph samples as realizations for device fitting.

use super::*;
use adventuresim_character_creator::{
    armor_frames::FitRegion,
    armor_gpu,
    armor_recipes::ParametricDesign,
    device_fit::{self, DevicePiece, FitBody, Realization},
};
use fabelgeist_armor::PartFrame;

/// Each body vertex's anatomical surface coordinate.
pub(super) fn vertex_texcoords(model: &BodyModel) -> Vec<[f32; 2]> {
    let character = &model.mhr.character;
    let mut uv = vec![[0.0; 2]; character.skin_weights.index.len()];
    for (face, uv_face) in character
        .mesh
        .faces
        .iter()
        .zip(&character.mesh.texcoord_faces)
    {
        for corner in 0..3 {
            uv[face[corner] as usize] = character.mesh.texcoords[uv_face[corner] as usize];
        }
    }
    uv
}

fn fit_body<'a>(model: &'a BodyModel, texcoords: &'a [[f32; 2]]) -> FitBody<'a> {
    let character = &model.mhr.character;
    FitBody {
        faces: &character.mesh.faces,
        texcoords,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    }
}

fn wearer(generated: &GeneratedCharacter) -> Realization<'_> {
    Realization {
        positions: &generated.positions,
        normals: &generated.normals,
        joints: &generated.global_joint_states,
        device: &generated.device,
    }
}

/// The part frame of `region` fitted to the wearer on the device.
pub(super) fn frame(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    region: FitRegion,
) -> Result<PartFrame> {
    let texcoords = vertex_texcoords(model);
    device_fit::frame(
        armor_gpu()?,
        &fit_body(model, &texcoords),
        &wearer(generated),
        region,
    )
}

/// Fit a recipe to the wearer and its morph samples on the device.
pub(super) fn fitted(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    morphs: &[ForearmMorphSample],
    design: &ParametricDesign,
    placement: &str,
    layers: &[&GeneratedArmor],
) -> Result<DevicePiece> {
    let texcoords = vertex_texcoords(model);
    let morphs = morphs
        .iter()
        .map(|sample| {
            (
                sample.name.as_str(),
                Realization {
                    positions: &sample.positions,
                    normals: &sample.normals,
                    joints: &sample.global_joint_states,
                    device: &sample.device,
                },
            )
        })
        .collect::<Vec<_>>();
    device_fit::fit_recipe(
        armor_gpu()?,
        &fit_body(model, &texcoords),
        &wearer(generated),
        &morphs,
        design,
        placement,
        layers,
    )
}
