use super::{MhrAssetDirectoryError, MhrAssetReadError};
use crate::character::{CharacterDecodeError, RigBlendShapeCount};
use crate::correctives::{CorrectiveArchiveRole, CorrectiveDecodeError};
use crate::model_def::ModelDefinitionError;
use fabelgeist_numpy_storage::ZipReadError;

#[derive(Debug)]
pub struct MhrModelLayoutError {
    pub shapes: RigBlendShapeCount,
}
impl RigBlendShapeCount {
    pub(super) fn admit_for_model(self) -> Result<(), MhrModelLayoutError> {
        if self == Self::from(super::NUM_BLEND_SHAPES) {
            Ok(())
        } else {
            Err(MhrModelLayoutError { shapes: self })
        }
    }
}
impl std::fmt::Display for MhrModelLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rig has {} blend shapes, expected {} identity plus {} expression",
            self.shapes,
            super::NUM_IDENTITY_BLEND_SHAPES,
            super::NUM_FACE_EXPRESSION_BLEND_SHAPES
        )
    }
}
impl std::error::Error for MhrModelLayoutError {}

#[derive(Debug)]
pub enum MhrLoadError {
    Directory(MhrAssetDirectoryError),
    Asset(MhrAssetReadError),
    Rig(CharacterDecodeError),
    CorrectiveArchive {
        role: CorrectiveArchiveRole,
        source: ZipReadError,
    },
    CorrectiveNetwork(CorrectiveDecodeError),
    Definition(ModelDefinitionError),
    BlendShapeColumns(ModelDefinitionError),
    ModelLayout(MhrModelLayoutError),
}
impl std::fmt::Display for MhrLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Directory(source) => source.fmt(f),
            Self::Asset(source) => source.fmt(f),
            Self::Rig(source) => write!(f, "loading the MHR FBX: {source}"),
            Self::CorrectiveArchive { role, source } => {
                let role = match role {
                    CorrectiveArchiveRole::Basis => "basis",
                    CorrectiveArchiveRole::Activation => "activation",
                };
                write!(f, "loading the pose-corrective {role}: {source}")
            }
            Self::CorrectiveNetwork(source) => {
                write!(f, "loading the pose-corrective network: {source}")
            }
            Self::Definition(source) => write!(f, "parsing the MHR model definition: {source}"),
            Self::BlendShapeColumns(source) => source.fmt(f),
            Self::ModelLayout(source) => source.fmt(f),
        }
    }
}
impl std::error::Error for MhrLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Directory(source) => Some(source),
            Self::Asset(source) => Some(source),
            Self::Rig(source) => Some(source),
            Self::CorrectiveArchive { source, .. } => Some(source),
            Self::CorrectiveNetwork(source) => Some(source),
            Self::Definition(source) | Self::BlendShapeColumns(source) => Some(source),
            Self::ModelLayout(source) => Some(source),
        }
    }
}
