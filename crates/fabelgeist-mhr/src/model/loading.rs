use super::assets::{MhrAsset, MhrAssetDirectory, MhrAssetReadError, ResourceMhrAssets};
use super::{Mhr, MhrLoadError, NUM_IDENTITY_BLEND_SHAPES, PoseParameterCount};
use crate::character::Character;
use crate::correctives::{CorrectiveArchiveRole, CorrectiveRigTopology, PoseCorrectives};
use crate::model_def::{BlendShapeParameterCount, ParameterTransform};
use crate::{MhrConfig, PoseCorrectivePolicy};
use burn::tensor::Device;
use fabelgeist_fs::{FileContents, FileText, ResourceLocator};
use fabelgeist_numpy_storage::Npz;

impl Mhr {
    /// Loads MHR through the admitted resource namespace.
    pub async fn from_uri(
        asset_dir: &ResourceLocator,
        config: MhrConfig,
        device: &Device,
    ) -> Result<Self, MhrLoadError> {
        let assets = ResourceMhrAssets::from(asset_dir);
        let fbx = assets
            .read(MhrAsset::Rig(config.lod))
            .await
            .map_err(MhrLoadError::Asset)?;
        let definition = FileText::try_from(
            assets
                .read(MhrAsset::ModelDefinition)
                .await
                .map_err(MhrLoadError::Asset)?,
        )
        .map_err(MhrAssetReadError::ModelDefinitionText)
        .map_err(MhrLoadError::Asset)?;
        let corrective_archives = match config.pose_correctives {
            PoseCorrectivePolicy::Enabled => Some((
                assets
                    .read(MhrAsset::CorrectiveActivation)
                    .await
                    .map_err(MhrLoadError::Asset)?,
                assets
                    .read(MhrAsset::CorrectiveBasis(config.lod))
                    .await
                    .map_err(MhrLoadError::Asset)?,
            )),
            PoseCorrectivePolicy::Disabled => None,
        };
        Self::from_asset_bytes(&fbx, &definition, corrective_archives, config, device)
    }

    /// Loads from a native asset directory or its parent.
    pub fn from_files(
        asset_dir: &MhrAssetDirectory,
        config: MhrConfig,
        device: &Device,
    ) -> Result<Self, MhrLoadError> {
        let assets = asset_dir.resolve().map_err(MhrLoadError::Directory)?;
        let fbx = assets
            .read(MhrAsset::Rig(config.lod))
            .map_err(MhrLoadError::Asset)?;
        let character = Character::from_fbx_bytes(&fbx).map_err(MhrLoadError::Rig)?;
        let definition = FileText::try_from(
            assets
                .read(MhrAsset::ModelDefinition)
                .map_err(MhrLoadError::Asset)?,
        )
        .map_err(MhrAssetReadError::ModelDefinitionText)
        .map_err(MhrLoadError::Asset)?;
        let correctives = match config.pose_correctives {
            PoseCorrectivePolicy::Enabled => {
                // Native loading retains its basis-before-activation admission.
                let basis = Npz::from_bytes(
                    assets
                        .read(MhrAsset::CorrectiveBasis(config.lod))
                        .map_err(MhrLoadError::Asset)?,
                )
                .map_err(
                    |source: fabelgeist_numpy_storage::ZipReadError| -> MhrLoadError {
                        MhrLoadError::CorrectiveArchive {
                            role: CorrectiveArchiveRole::Basis,
                            source,
                        }
                    },
                )?;
                let activation = Npz::from_bytes(
                    assets
                        .read(MhrAsset::CorrectiveActivation)
                        .map_err(MhrLoadError::Asset)?,
                )
                .map_err(
                    |source: fabelgeist_numpy_storage::ZipReadError| -> MhrLoadError {
                        MhrLoadError::CorrectiveArchive {
                            role: CorrectiveArchiveRole::Activation,
                            source,
                        }
                    },
                )?;
                PoseCorrectives::from_archives(
                    activation,
                    basis,
                    CorrectiveRigTopology::from(&character),
                    device,
                )
                .map_err(MhrLoadError::CorrectiveNetwork)?
            }
            PoseCorrectivePolicy::Disabled => None,
        };
        Self::from_loaded_assets(character, &definition, correctives, device)
    }

    /// Loads from the same nominal FBX/model/NPZ payloads as the provider paths.
    pub fn from_asset_bytes(
        fbx: &FileContents,
        definition: &FileText,
        corrective_archives: Option<(FileContents, FileContents)>,
        config: MhrConfig,
        device: &Device,
    ) -> Result<Self, MhrLoadError> {
        let character = Character::from_fbx_bytes(fbx).map_err(MhrLoadError::Rig)?;
        let correctives = match config.pose_correctives {
            PoseCorrectivePolicy::Enabled => {
                let (activation, basis) = corrective_archives
                    .ok_or(MhrAssetReadError::MissingCorrectiveArchives)
                    .map_err(MhrLoadError::Asset)?;
                PoseCorrectives::from_bytes(
                    activation,
                    basis,
                    CorrectiveRigTopology::from(&character),
                    device,
                )
                .map_err(MhrLoadError::CorrectiveNetwork)?
            }
            PoseCorrectivePolicy::Disabled => None,
        };
        Self::from_loaded_assets(character, definition, correctives, device)
    }

    fn from_loaded_assets(
        character: Character,
        definition: &FileText,
        correctives: Option<PoseCorrectives>,
        device: &Device,
    ) -> Result<Self, MhrLoadError> {
        let mut parameter_transform =
            ParameterTransform::from_definition(definition, &character.skeleton)
                .map_err(MhrLoadError::Definition)?;
        let num_model_parameters = PoseParameterCount::from(parameter_transform.num_parameters());
        // Momentum appends identity blend-shape parameters to the character.
        parameter_transform
            .append_blend_shapes(BlendShapeParameterCount::from(NUM_IDENTITY_BLEND_SHAPES))
            .map_err(MhrLoadError::BlendShapeColumns)?;
        Self::new(
            character,
            parameter_transform,
            num_model_parameters,
            correctives,
            device,
        )
        .map_err(MhrLoadError::ModelLayout)
    }
}

#[cfg(test)]
mod tests;
