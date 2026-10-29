//! Native tensor construction from the admitted MHR host aggregates.

use super::{
    Mhr, NUM_BLEND_SHAPES, NUM_FACE_EXPRESSION_BLEND_SHAPES, NUM_IDENTITY_BLEND_SHAPES, rig,
};
use crate::character::{Character, PARAMETERS_PER_JOINT};
use crate::correctives::PoseCorrectives;
use crate::model_def::ParameterTransform;
use anyhow::{Result, bail};
use burn::tensor::{Device, Tensor, TensorData};
use fabelgeist_numpy_storage::NpyDimension;

/// Sparse i32 index words and coefficients for Burn's influence kernel.
struct SparseSkinUpload {
    joints: Vec<i32>,
    vertices: Vec<i32>,
    weights: Vec<f32>,
}

struct RestRigTensors {
    translations: Tensor<3>,
    prerotations: Tensor<3>,
    inverse_bind_pose: Tensor<3>,
    base_shape: Tensor<2>,
}

struct BlendBasisTensors {
    identity: Tensor<2>,
    expression: Tensor<2>,
}

impl Mhr {
    pub(super) fn new(
        character: Character,
        parameter_transform: ParameterTransform,
        num_model_parameters: NpyDimension,
        correctives: Option<PoseCorrectives>,
        device: &Device,
    ) -> Result<Self> {
        let joints = character.skeleton.names.joint_count();
        if character.blend_shapes.len() != NUM_BLEND_SHAPES {
            bail!(
                "rig has {} blend shapes, expected {NUM_IDENTITY_BLEND_SHAPES} identity plus \
                 {NUM_FACE_EXPRESSION_BLEND_SHAPES} expression",
                character.blend_shapes.len()
            );
        }

        // Transposed so a forward pass is `parameters @ transform`.
        let transform = parameter_transform
            .transform
            .tensor_transpose(num_model_parameters)?;

        let offsets = parameter_transform
            .offsets
            .iter()
            .any(|offset| *offset != 0.0)
            .then(|| {
                Tensor::from_data(
                    TensorData::new(
                        parameter_transform.offsets.tensor_values(),
                        [1, joints.native_joint_count() * PARAMETERS_PER_JOINT],
                    ),
                    device,
                )
            });

        let fk_levels = rig::fk_level_tensors(&character, device)?;

        let basis = BlendBasisTensors::from_character(&character, device);

        let skin = SparseSkinUpload::from_character(&character)?;
        let influences = skin.weights.len();

        let face_corners: Vec<i32> = character
            .mesh
            .faces
            .iter()
            .flatten()
            .map(|corner| i32::try_from(*corner))
            .collect::<Result<_, _>>()?;
        let num_corners = face_corners.len();

        let rest = RestRigTensors::from_character(&character, device);
        Ok(Self {
            device: device.clone(),
            transform: Tensor::from_data(
                TensorData::new(
                    transform,
                    [
                        usize::from(num_model_parameters),
                        joints.native_joint_count() * PARAMETERS_PER_JOINT,
                    ],
                ),
                device,
            ),
            offsets,
            num_model_parameters,
            joint_translation_offsets: rest.translations,
            joint_prerotations: rest.prerotations,
            inverse_bind_pose: rest.inverse_bind_pose,
            fk_levels,
            identity_basis: basis.identity,
            expression_basis: basis.expression,
            base_shape: rest.base_shape,
            face_corners: Tensor::from_data(TensorData::new(face_corners, [num_corners]), device),
            skin_joint_indices: Tensor::from_data(
                TensorData::new(skin.joints, [influences]),
                device,
            ),
            skin_vertex_indices: Tensor::from_data(
                TensorData::new(skin.vertices, [influences]),
                device,
            ),
            skin_weights: Tensor::from_data(
                TensorData::new(skin.weights, [1, influences, 1]),
                device,
            ),
            correctives,
            character,
            parameter_transform,
        })
    }
}

impl SparseSkinUpload {
    fn from_character(character: &Character) -> Result<Self> {
        // Drop zero influences: MHR averages under three joints per vertex.
        let mut skin_joint_indices = Vec::new();
        let mut skin_vertex_indices = Vec::new();
        let mut skin_weights = Vec::new();
        // Encode the admitted vertex/joint roles into Burn's sparse i32 buffers.
        for (vertex, influences) in character.skin_weights.indexed() {
            for (joint, weight) in influences.joints.into_iter().zip(influences.weights) {
                if weight > 1e-5 {
                    skin_joint_indices.push(i32::try_from(usize::from(joint))?);
                    skin_vertex_indices.push(i32::try_from(vertex.native_vertex_index())?);
                    skin_weights.push(weight);
                }
            }
        }
        Ok(Self {
            joints: skin_joint_indices,
            vertices: skin_vertex_indices,
            weights: skin_weights,
        })
    }
}

impl BlendBasisTensors {
    fn from_character(character: &Character, device: &Device) -> Self {
        let vertices = character.mesh.vertex_count();
        let basis = &character.blend_shapes.vectors;
        let identity_basis = Tensor::from_data(
            TensorData::new(
                basis[..NUM_IDENTITY_BLEND_SHAPES * vertices.native_vertex_count() * 3].to_vec(),
                [
                    NUM_IDENTITY_BLEND_SHAPES,
                    vertices.native_vertex_count() * 3,
                ],
            ),
            device,
        );
        let expression_basis = Tensor::from_data(
            TensorData::new(
                basis[NUM_IDENTITY_BLEND_SHAPES * vertices.native_vertex_count() * 3..].to_vec(),
                [
                    NUM_FACE_EXPRESSION_BLEND_SHAPES,
                    vertices.native_vertex_count() * 3,
                ],
            ),
            device,
        );

        Self {
            identity: identity_basis,
            expression: expression_basis,
        }
    }
}

impl RestRigTensors {
    fn from_character(character: &Character, device: &Device) -> Self {
        let joints = character.skeleton.names.joint_count();
        let vertices = character.mesh.vertex_count();
        Self {
            translations: Tensor::from_data(
                TensorData::new(
                    character
                        .skeleton
                        .translation_offsets
                        .as_native_slice()
                        .concat(),
                    [1, joints.native_joint_count(), 3],
                ),
                device,
            ),
            prerotations: Tensor::from_data(
                TensorData::new(
                    character.skeleton.prerotations.as_native_slice().concat(),
                    [1, joints.native_joint_count(), 4],
                ),
                device,
            ),
            inverse_bind_pose: Tensor::from_data(
                TensorData::new(
                    character.inverse_bind_pose.as_native_slice().concat(),
                    [1, joints.native_joint_count(), 8],
                ),
                device,
            ),
            base_shape: Tensor::from_data(
                TensorData::new(
                    character.mesh.vertices.as_native_slice().concat(),
                    [1, vertices.native_vertex_count() * 3],
                ),
                device,
            ),
        }
    }
}
