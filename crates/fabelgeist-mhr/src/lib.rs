//! Native Burn implementation of MHR, the Momentum Human Rig.
//!
//! MHR is a parametric 3D body model with 45 identity coefficients, 204 pose
//! and scale parameters, 72 facial expression coefficients, and a non-linear
//! pose-corrective network, published across seven levels of detail.
//!
//! Everything is loaded straight from the released assets — the binary FBX
//! rig, the `.model` parameter transform, and the `.npz` correctives — with no
//! Python, no FBX SDK, and no momentum runtime. The forward pass is Burn
//! tensors end to end.
//!
//! ```no_run
//! # fn main() -> Result<(), fabelgeist_mhr::MhrLoadError> {
//! use burn::tensor::{Device, Tensor};
//! use fabelgeist_mhr::{Mhr, MhrAssetDirectory, MhrConfig, ModelBatchSize, NUM_IDENTITY_BLEND_SHAPES};
//!
//! let device = Device::default();
//! let assets = MhrAssetDirectory::from(std::path::PathBuf::from("target/mhr-assets/assets"));
//! let model = Mhr::from_files(&assets, MhrConfig::default(), &device)?;
//!
//! let identity = Tensor::zeros([1, NUM_IDENTITY_BLEND_SHAPES], &device);
//! let pose = model.zero_parameters(ModelBatchSize::from(1));
//! let output = model.forward(identity, pose, None).expect("admitted identity and pose dimensions");
//! # let _ = output;
//! # Ok(())
//! # }
//! ```
//!
//! Reference implementation: <https://github.com/facebookresearch/MHR>.

pub mod character;
pub mod config;
pub mod correctives;
pub mod math;
pub mod model;
pub mod model_def;
pub mod skel_state;

pub use character::{
    BlendShapes, Character, CharacterDecodeError, Mesh, PolygonCornerCount, RigArrayCount,
    RigBlendShapeCount, RigMeshVertexOrdinal, RigObjectContext, Skeleton, SkinWeights,
};
pub use config::{
    CharacterLod, CharacterLodError, CharacterLodViolation, MeshVertexCount, MhrConfig,
    PoseCorrectiveAvailability, PoseCorrectivePolicy,
};
pub use correctives::{CorrectiveDecodeError, CorrectiveRigTopology, PoseCorrectives};
pub use model::{
    ExpressionCoefficientCount, IdentityCoefficientCount, Mhr, MhrAsset, MhrAssetDirectory,
    MhrAssetDirectoryError, MhrAssetReadError, MhrEvaluationError, MhrLoadError,
    MhrModelLayoutError, MhrOutput, ModelBatchSize, NUM_BLEND_SHAPES,
    NUM_FACE_EXPRESSION_BLEND_SHAPES, NUM_IDENTITY_BLEND_SHAPES, PoseParameterCount,
};
pub use model_def::{
    BlendShapeParameterCount, JointParameterChannel, JointParameterRow, ModelDefinitionError,
    ModelParameterCount, ModelParameterIndex, ModelParameterName, ParameterSetName,
    ParameterTransform,
};
