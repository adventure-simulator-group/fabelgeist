//! Authored asset roles and native asset-directory selection.

use crate::CharacterLod;
use fabelgeist_fs::{EntryName, FileContents, ResourceIoError, ResourceLocator};
use std::path::PathBuf;

mod error;
pub use error::{MhrAssetDirectoryError, MhrAssetReadError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MhrAsset {
    Rig(CharacterLod),
    ModelDefinition,
    CorrectiveActivation,
    CorrectiveBasis(CharacterLod),
}
impl MhrAsset {
    pub fn file_name(self) -> EntryName {
        let spelling = match self {
            Self::Rig(CharacterLod::Detailed) => "lod4.fbx",
            Self::Rig(CharacterLod::Reduced) => "lod5.fbx",
            Self::Rig(CharacterLod::Minimal) => "lod6.fbx",
            Self::ModelDefinition => "compact_v6_1.model",
            Self::CorrectiveActivation => "corrective_activation.npz",
            Self::CorrectiveBasis(CharacterLod::Detailed) => "corrective_blendshapes_lod4.npz",
            Self::CorrectiveBasis(CharacterLod::Reduced) => "corrective_blendshapes_lod5.npz",
            Self::CorrectiveBasis(CharacterLod::Minimal) => "corrective_blendshapes_lod6.npz",
        };
        EntryName::try_from(spelling).expect("authored MHR asset names are single children")
    }
}

/// A native asset namespace, admitted from a caller's directory address.
/// Selection accepts that directory or its `assets` child and retains provider
/// inspection errors rather than reporting them as missing assets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MhrAssetDirectory {
    path: PathBuf,
}
impl From<PathBuf> for MhrAssetDirectory {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}
impl std::fmt::Display for MhrAssetDirectory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.display().fmt(formatter)
    }
}
enum DefinitionPresence {
    Present,
    Absent,
}
impl MhrAssetDirectory {
    pub(super) fn resolve(&self) -> Result<Self, MhrAssetDirectoryError> {
        if let DefinitionPresence::Present = self.definition_presence()? {
            return Ok(self.clone());
        }
        let nested = Self::from(self.path.join("assets"));
        if let DefinitionPresence::Present = nested.definition_presence()? {
            return Ok(nested);
        }
        Err(MhrAssetDirectoryError::Missing {
            directory: self.clone(),
            nested,
        })
    }
    fn definition_presence(&self) -> Result<DefinitionPresence, MhrAssetDirectoryError> {
        let name = MhrAsset::ModelDefinition.file_name();
        match std::fs::metadata(self.path.join(name.as_ref())) {
            Ok(metadata) if metadata.is_file() => Ok(DefinitionPresence::Present),
            Ok(_) => Ok(DefinitionPresence::Absent),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                Ok(DefinitionPresence::Absent)
            }
            Err(source) => Err(MhrAssetDirectoryError::Inspect {
                directory: self.clone(),
                source,
            }),
        }
    }
    pub(super) fn read(&self, asset: MhrAsset) -> Result<FileContents, MhrAssetReadError> {
        let name = asset.file_name();
        std::fs::read(self.path.join(name.as_ref()))
            .map(FileContents::from)
            .map_err(|source: std::io::Error| -> MhrAssetReadError {
                MhrAssetReadError::Native {
                    directory: self.clone(),
                    asset,
                    source,
                }
            })
    }
}

pub(super) struct ResourceMhrAssets {
    directory: ResourceLocator,
}
impl From<&ResourceLocator> for ResourceMhrAssets {
    fn from(directory: &ResourceLocator) -> Self {
        Self {
            directory: directory.clone(),
        }
    }
}
impl ResourceMhrAssets {
    pub(super) async fn read(&self, asset: MhrAsset) -> Result<FileContents, MhrAssetReadError> {
        let base = self.directory.as_ref().trim_end_matches(['/', '\\']);
        let name = asset.file_name();
        let direct = ResourceLocator::try_from(format!("{base}/{name}").as_str())
            .map_err(MhrAssetReadError::Address)?;
        match direct.read().await {
            Ok(contents) => Ok(contents),
            Err(direct) => {
                let nested = ResourceLocator::try_from(format!("{base}/assets/{name}").as_str())
                    .map_err(MhrAssetReadError::Address)?;
                nested
                    .read()
                    .await
                    .map_err(|nested: ResourceIoError| -> MhrAssetReadError {
                        MhrAssetReadError::Lookup {
                            asset,
                            direct: Box::new(direct),
                            nested: Box::new(nested),
                        }
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests;
