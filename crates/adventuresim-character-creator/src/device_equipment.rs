//! The studio's body and morph samples as realizations for device fitting.

use super::*;
use adventuresim_character_creator::armor_frames::FittingPlacement;
use adventuresim_character_creator::{
    armor_frames::FitRegion,
    armor_gpu,
    armor_recipes::ParametricDesign,
    device_fit::{self, DevicePiece, FitBody, Realization},
};
use fabelgeist_armor::PartFrame;

/// Each body vertex's anatomical surface coordinate.
pub(super) fn vertex_texcoords(
    model: &BodyModel,
) -> Result<fabelgeist_mhr::character::MeshVertexTable<[f32; 2]>> {
    let character = &model.mhr.character;
    let mut uv =
        fabelgeist_mhr::character::MeshVertexTable::filled(model.mhr.num_vertices(), [0.0; 2]);
    for (face, uv_face) in character
        .mesh
        .faces
        .iter()
        .zip(&character.mesh.texcoord_faces)
    {
        for corner in 0..3 {
            let vertex = fabelgeist_mhr::character::MeshVertexIndex::from_native_vertex_index(
                face[corner] as usize,
            );
            let coordinate = uv
                .get_mut(vertex)
                .context("body face references a missing vertex")?;
            *coordinate = *character
                .mesh
                .texcoords
                .get(uv_face[corner] as usize)
                .context("body face references a missing texture coordinate")?;
        }
    }
    Ok(uv)
}

fn fit_body<'a>(model: &'a BodyModel, texcoords: &'a [[f32; 2]]) -> FitBody<'a> {
    let character = &model.mhr.character;
    FitBody {
        faces: &character.mesh.faces,
        texcoords,
        joint_indices: character.skin_weights.indices(),
        joint_weights: character.skin_weights.weights().as_native_slice(),
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
    let texcoords = vertex_texcoords(model)?;
    device_fit::frame(
        armor_gpu()?,
        &fit_body(model, texcoords.as_native_slice()),
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
    placement: FittingPlacement,
    layers: &[&GeneratedArmor],
) -> Result<DevicePiece> {
    let texcoords = vertex_texcoords(model)?;
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
        &fit_body(model, texcoords.as_native_slice()),
        &wearer(generated),
        &morphs,
        design,
        placement,
        layers,
    )
}
