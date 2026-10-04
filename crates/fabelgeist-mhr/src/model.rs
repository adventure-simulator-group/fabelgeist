//! The MHR body model: identity, pose and expression parameters in, posed
//! mesh vertices and a skeleton state out.

mod assets;
mod error;
mod input;
mod loading;
pub use assets::{MhrAsset, MhrAssetDirectory, MhrAssetDirectoryError, MhrAssetReadError};
pub use error::{MhrLoadError, MhrModelLayoutError};
pub use input::{
    ExpressionCoefficientCount, IdentityCoefficientCount, MhrEvaluationError, ModelBatchSize,
    PoseParameterCount,
};
use input::{ExpressionLayout, IdentityLayout, IdentityRows, PoseLayout};

use burn::tensor::ops::IndexingUpdateOp;
use burn::tensor::{Device, Int, Tensor, TensorData};

use crate::character::{Character, PARAMETERS_PER_JOINT};
use crate::correctives::PoseCorrectives;
use crate::model_def::ParameterTransform;
use crate::skel_state;
use crate::{PoseCorrectiveAvailability, PoseCorrectivePolicy};

/// Shape coefficients: 20 body, 20 head, 5 hands.
pub const NUM_IDENTITY_BLEND_SHAPES: usize = 45;
/// Facial expression coefficients.
pub const NUM_FACE_EXPRESSION_BLEND_SHAPES: usize = 72;
/// Total blend shapes carried by an MHR rig.
pub const NUM_BLEND_SHAPES: usize = NUM_IDENTITY_BLEND_SHAPES + NUM_FACE_EXPRESSION_BLEND_SHAPES;

/// One forward pass.
pub struct MhrOutput {
    /// Posed vertices in centimetres, `[batch, vertices, 3]`.
    pub vertices: Tensor<3>,
    /// Unit vertex normals of the posed mesh, `[batch, vertices, 3]`.
    ///
    /// Recomputed from the deformed vertices rather than skinned from the rest
    /// pose, because blend shapes and the pose correctives change the surface
    /// itself, not just its rigid frame.
    pub normals: Tensor<3>,
    /// Global joint transforms `[tx, ty, tz, qx, qy, qz, qw, s]`, `[batch, joints, 8]`.
    pub skeleton_state: Tensor<3>,
}

/// The MHR body model.
pub struct Mhr {
    /// Topology and names, kept on the host for lookups and the normal pass.
    pub character: Character,
    parameter_transform: ParameterTransform,
    device: Device,

    /// `[model parameters, joints * 7]`, transposed for a right-hand matmul.
    transform: Tensor<2>,
    /// Constant joint-parameter offsets, or `None` when the rig has none.
    offsets: Option<Tensor<2>>,
    num_model_parameters: PoseParameterCount,

    /// `[1, joints, 3]` and `[1, joints, 4]`.
    joint_translation_offsets: Tensor<3>,
    joint_prerotations: Tensor<3>,
    /// `[1, joints, 8]`.
    inverse_bind_pose: Tensor<3>,
    /// Levels of the forward-kinematics prefix scan.
    fk_levels: Vec<(Tensor<1, Int>, Tensor<1, Int>)>,

    /// `[45, vertices * 3]` and `[72, vertices * 3]`.
    identity_basis: Tensor<2>,
    expression_basis: Tensor<2>,
    /// `[1, vertices * 3]`.
    base_shape: Tensor<2>,

    /// The three corners of every triangle, flattened, `[faces * 3]`.
    face_corners: Tensor<1, Int>,

    /// Flattened non-zero skinning influences.
    skin_joint_indices: Tensor<1, Int>,
    skin_vertex_indices: Tensor<1, Int>,
    /// `[1, influences, 1]`.
    skin_weights: Tensor<3>,

    correctives: Option<PoseCorrectives>,
}

impl Mhr {
    /// Admitted host transform, immutable after its device projection is built.
    pub fn parameter_transform(&self) -> &ParameterTransform {
        &self.parameter_transform
    }

    fn new(
        character: Character,
        parameter_transform: ParameterTransform,
        num_model_parameters: PoseParameterCount,
        correctives: Option<PoseCorrectives>,
        device: &Device,
    ) -> Result<Self, MhrModelLayoutError> {
        let joints = character.skeleton.len();
        let vertices = character.mesh.vertices.len();

        character.blend_shapes.len().admit_for_model()?;

        // Transposed so a forward pass is `parameters @ transform`.
        let columns = usize::from(parameter_transform.num_parameters());
        let model_columns = usize::from(num_model_parameters);
        let mut transform = vec![0.0f32; model_columns * joints * PARAMETERS_PER_JOINT];
        for row in 0..joints * PARAMETERS_PER_JOINT {
            for column in 0..model_columns {
                transform[column * joints * PARAMETERS_PER_JOINT + row] =
                    parameter_transform.transform[row * columns + column];
            }
        }

        let offsets = parameter_transform
            .offsets
            .iter()
            .any(|offset| *offset != 0.0)
            .then(|| {
                Tensor::from_data(
                    TensorData::new(
                        parameter_transform.offsets.clone(),
                        [1, joints * PARAMETERS_PER_JOINT],
                    ),
                    device,
                )
            });

        let fk_levels = skel_state::prefix_multiplication_levels(&character.skeleton.parents)
            .into_iter()
            .map(|(source, target)| {
                let len = source.len();
                (
                    Tensor::from_data(TensorData::new(source, [len]), device),
                    Tensor::from_data(TensorData::new(target, [len]), device),
                )
            })
            .collect();

        let basis = &character.blend_shapes.vectors;
        let identity_basis = Tensor::from_data(
            TensorData::new(
                basis[..NUM_IDENTITY_BLEND_SHAPES * vertices * 3].to_vec(),
                [NUM_IDENTITY_BLEND_SHAPES, vertices * 3],
            ),
            device,
        );
        let expression_basis = Tensor::from_data(
            TensorData::new(
                basis[NUM_IDENTITY_BLEND_SHAPES * vertices * 3..].to_vec(),
                [NUM_FACE_EXPRESSION_BLEND_SHAPES, vertices * 3],
            ),
            device,
        );

        // Drop zero influences: MHR averages under three joints per vertex.
        let mut skin_joint_indices = Vec::new();
        let mut skin_vertex_indices = Vec::new();
        let mut skin_weights = Vec::new();
        for (vertex, (indices, weights)) in character
            .skin_weights
            .index
            .iter()
            .zip(&character.skin_weights.weight)
            .enumerate()
        {
            for (joint, weight) in indices.iter().zip(weights) {
                if *weight > 1e-5 {
                    skin_joint_indices.push(*joint as i32);
                    skin_vertex_indices.push(vertex as i32);
                    skin_weights.push(*weight);
                }
            }
        }
        let influences = skin_weights.len();

        let face_corners: Vec<i32> = character
            .mesh
            .faces
            .iter()
            .flatten()
            .map(|corner| *corner as i32)
            .collect();
        let num_corners = face_corners.len();

        Ok(Self {
            device: device.clone(),
            transform: Tensor::from_data(
                TensorData::new(transform, [model_columns, joints * PARAMETERS_PER_JOINT]),
                device,
            ),
            offsets,
            num_model_parameters,
            joint_translation_offsets: Tensor::from_data(
                TensorData::new(
                    character.skeleton.translation_offsets.concat(),
                    [1, joints, 3],
                ),
                device,
            ),
            joint_prerotations: Tensor::from_data(
                TensorData::new(character.skeleton.prerotations.concat(), [1, joints, 4]),
                device,
            ),
            inverse_bind_pose: Tensor::from_data(
                TensorData::new(character.inverse_bind_pose.concat(), [1, joints, 8]),
                device,
            ),
            fk_levels,
            identity_basis,
            expression_basis,
            base_shape: Tensor::from_data(
                TensorData::new(character.mesh.vertices.concat(), [1, vertices * 3]),
                device,
            ),
            face_corners: Tensor::from_data(TensorData::new(face_corners, [num_corners]), device),
            skin_joint_indices: Tensor::from_data(
                TensorData::new(skin_joint_indices, [influences]),
                device,
            ),
            skin_vertex_indices: Tensor::from_data(
                TensorData::new(skin_vertex_indices, [influences]),
                device,
            ),
            skin_weights: Tensor::from_data(
                TensorData::new(skin_weights, [1, influences, 1]),
                device,
            ),
            correctives,
            character,
            parameter_transform,
        })
    }

    pub fn num_joints(&self) -> usize {
        self.character.skeleton.len()
    }

    pub fn num_vertices(&self) -> usize {
        self.character.mesh.vertices.len()
    }

    /// Number of pose/scale parameters the model takes, excluding the blend
    /// shape coefficients momentum appends to the parameter vector.
    pub fn num_model_parameters(&self) -> PoseParameterCount {
        self.num_model_parameters
    }

    pub fn pose_corrective_availability(&self) -> PoseCorrectiveAvailability {
        match self.correctives {
            Some(_) => PoseCorrectiveAvailability::Available,
            None => PoseCorrectiveAvailability::Unavailable,
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Zero parameters for a batch, i.e. the rest pose.
    pub fn zero_parameters(&self, batch: ModelBatchSize) -> Tensor<2> {
        Tensor::zeros(
            [usize::from(batch), usize::from(self.num_model_parameters)],
            &self.device,
        )
    }

    /// Runs the model.
    ///
    /// * `identity` — `[batch, 45]` shape coefficients (a single row is broadcast).
    /// * `model_parameters` — `[batch, 204]` pose and scale parameters.
    /// * `expression` — `[batch, 72]` facial expression coefficients, optional.
    pub fn forward(
        &self,
        identity: Tensor<2>,
        model_parameters: Tensor<2>,
        expression: Option<Tensor<2>>,
    ) -> std::result::Result<MhrOutput, MhrEvaluationError> {
        self.forward_with(
            identity,
            model_parameters,
            expression,
            PoseCorrectivePolicy::Enabled,
        )
    }

    /// As [`Mhr::forward`], but able to skip the pose correctives.
    pub fn forward_with(
        &self,
        identity: Tensor<2>,
        model_parameters: Tensor<2>,
        expression: Option<Tensor<2>>,
        apply_correctives: PoseCorrectivePolicy,
    ) -> std::result::Result<MhrOutput, MhrEvaluationError> {
        let pose_layout = PoseLayout::from(model_parameters.dims());
        let batch = pose_layout.batch;
        let joints = self.num_joints();
        let vertices = self.num_vertices();
        let identity_rows = IdentityLayout::from(identity.dims()).admit(batch)?;
        pose_layout.admit(self.num_model_parameters)?;

        // Rest shape: mean plus identity and expression offsets.
        let identity = match identity_rows {
            IdentityRows::Exact => identity,
            IdentityRows::Broadcast => {
                identity.expand([usize::from(batch), NUM_IDENTITY_BLEND_SHAPES])
            }
        };
        let mut rest = self.base_shape.clone() + identity.matmul(self.identity_basis.clone());
        if let Some(expression) = expression {
            ExpressionLayout::from(expression.dims()).admit(batch)?;
            rest = rest + expression.matmul(self.expression_basis.clone());
        }

        // Model parameters drive joint parameters, seven channels per joint.
        //
        // Deliberately a broadcast product and reduction rather than a matmul:
        // the CUDA backend promotes f32 matmuls to tf32, whose 10-bit mantissa
        // costs about 2e-4 relative accuracy. That is invisible in a blend
        // shape but not here, where every joint angle and translation is
        // amplified along the kinematic chain. The tensor is 204 x 889, so the
        // exact form is cheap.
        let mut joint_parameters = (model_parameters.unsqueeze_dim::<3>(2)
            * self.transform.clone().unsqueeze::<3>())
        .sum_dim(1)
        .reshape([usize::from(batch), joints * PARAMETERS_PER_JOINT]);
        if let Some(offsets) = &self.offsets {
            joint_parameters = joint_parameters + offsets.clone();
        }
        let joint_parameters =
            joint_parameters.reshape([usize::from(batch), joints, PARAMETERS_PER_JOINT]);

        let skeleton_state = self.skeleton_state(joint_parameters.clone());

        let mut rest = rest.reshape([usize::from(batch), vertices, 3]);
        if apply_correctives == PoseCorrectivePolicy::Enabled
            && let Some(correctives) = &self.correctives
        {
            rest = rest + correctives.forward(joint_parameters);
        }

        let vertices = self.skin(skeleton_state.clone(), rest);

        Ok(MhrOutput {
            normals: self.vertex_normals(vertices.clone()),
            vertices,
            skeleton_state,
        })
    }

    /// Joint parameters `[batch, joints, 7]` to global skeleton states.
    fn skeleton_state(&self, joint_parameters: Tensor<3>) -> Tensor<3> {
        let translation =
            joint_parameters.clone().narrow(2, 0, 3) + self.joint_translation_offsets.clone();
        let rotation = skel_state::quaternion_multiply(
            self.joint_prerotations.clone(),
            skel_state::euler_xyz_to_quaternion(joint_parameters.clone().narrow(2, 3, 3)),
        );
        // momentum stores scale as a power of two.
        let scale = (joint_parameters.narrow(2, 6, 1) * std::f32::consts::LN_2).exp();

        let mut state = Tensor::cat(vec![translation, rotation, scale], 2);
        for (source, target) in &self.fk_levels {
            let parent = state.clone().select(1, target.clone());
            let child = state.clone().select(1, source.clone());
            // An overwrite expressed as an accumulation: source indices are
            // unique within a level, and `Assign` has no backend kernel here.
            let delta = skel_state::multiply(parent, child.clone()) - child;
            state = state.select_assign(1, source.clone(), delta, IndexingUpdateOp::Add);
        }
        state
    }

    /// Area-weighted vertex normals of a posed mesh, `[batch, vertices, 3]`.
    ///
    /// Each triangle contributes its unnormalized cross product, whose length
    /// is twice the triangle's area, to all three of its corners. That is the
    /// weighting `THREE.BufferGeometry.computeVertexNormals` uses, which is
    /// what the reference web viewer shades MHR with.
    fn vertex_normals(&self, vertices: Tensor<3>) -> Tensor<3> {
        let batch = vertices.dims()[0];
        let faces = self.character.mesh.faces.len();

        let corners = vertices
            .select(1, self.face_corners.clone())
            .reshape([batch, faces, 3, 3]);
        let corner = |index: usize| {
            corners
                .clone()
                .narrow(2, index, 1)
                .reshape([batch, faces, 3])
        };
        let (first, second, third) = (corner(0), corner(1), corner(2));
        let (u, v) = (second - first.clone(), third - first);

        let axis = |t: &Tensor<3>, index: usize| t.clone().narrow(2, index, 1);
        let cross = Tensor::cat(
            vec![
                axis(&u, 1) * axis(&v, 2) - axis(&u, 2) * axis(&v, 1),
                axis(&u, 2) * axis(&v, 0) - axis(&u, 0) * axis(&v, 2),
                axis(&u, 0) * axis(&v, 1) - axis(&u, 1) * axis(&v, 0),
            ],
            2,
        );

        // The same face normal lands on each of the face's three corners.
        let contribution = cross
            .unsqueeze_dim::<4>(2)
            .expand([batch, faces, 3, 3])
            .reshape([batch, faces * 3, 3]);
        let accumulated = Tensor::zeros([batch, self.num_vertices(), 3], &self.device)
            .select_assign(
                1,
                self.face_corners.clone(),
                contribution,
                IndexingUpdateOp::Add,
            );

        // A vertex no triangle references keeps a zero normal, as it does in
        // three.js; nothing rasterizes it, so there is no direction to invent.
        let length = accumulated
            .clone()
            .powi_scalar(2)
            .sum_dim(2)
            .sqrt()
            .clamp_min(1e-12);
        accumulated / length
    }

    /// Linear blend skinning of `rest` `[batch, vertices, 3]`.
    fn skin(&self, skeleton_state: Tensor<3>, rest: Tensor<3>) -> Tensor<3> {
        let batch = rest.dims()[0];
        let joint_state = skel_state::multiply(skeleton_state, self.inverse_bind_pose.clone());

        let transforms = joint_state.select(1, self.skin_joint_indices.clone());
        let points = rest.clone().select(1, self.skin_vertex_indices.clone());
        let deformed =
            skel_state::transform_points(&transforms, points) * self.skin_weights.clone();

        Tensor::zeros(rest.dims(), &self.device)
            .select_assign(
                1,
                self.skin_vertex_indices.clone(),
                deformed,
                IndexingUpdateOp::Add,
            )
            .reshape([batch, self.num_vertices(), 3])
    }
}
