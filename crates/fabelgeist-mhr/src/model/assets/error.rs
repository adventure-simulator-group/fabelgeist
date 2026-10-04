use super::{MhrAsset, MhrAssetDirectory};
use fabelgeist_fs::{FileTextError, ResourceIoError, ResourceLocatorError};

#[derive(Debug)]
pub enum MhrAssetReadError {
    Address(ResourceLocatorError),
    Lookup {
        asset: MhrAsset,
        direct: Box<ResourceIoError>,
        nested: Box<ResourceIoError>,
    },
    Native {
        directory: MhrAssetDirectory,
        asset: MhrAsset,
        source: std::io::Error,
    },
    ModelDefinitionText(FileTextError),
    MissingCorrectiveArchives,
}
impl std::fmt::Display for MhrAssetReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Address(source) => source.fmt(formatter),
            Self::Lookup {
                asset,
                direct,
                nested,
            } => write!(
                formatter,
                "could not read MHR asset {}: {direct}; {nested}",
                asset.file_name()
            ),
            Self::Native {
                directory,
                asset,
                source,
            } => write!(
                formatter,
                "could not read MHR asset {} in {directory}: {source}",
                asset.file_name()
            ),
            Self::ModelDefinitionText(source) => {
                write!(formatter, "MHR model definition is not UTF-8: {source}")
            }
            Self::MissingCorrectiveArchives => formatter
                .write_str("pose correctives requested but their NPZ archives were not provided"),
        }
    }
}
impl std::error::Error for MhrAssetReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Address(source) => Some(source),
            Self::Lookup { nested, .. } => Some(nested.as_ref()),
            Self::Native { source, .. } => Some(source),
            Self::ModelDefinitionText(source) => Some(source),
            Self::MissingCorrectiveArchives => None,
        }
    }
}

#[derive(Debug)]
pub enum MhrAssetDirectoryError {
    Inspect {
        directory: MhrAssetDirectory,
        source: std::io::Error,
    },
    Missing {
        directory: MhrAssetDirectory,
        nested: MhrAssetDirectory,
    },
}
impl std::fmt::Display for MhrAssetDirectoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inspect { directory, source } => write!(
                formatter,
                "could not inspect MHR model definition in {directory}: {source}"
            ),
            Self::Missing { directory, nested } => write!(
                formatter,
                "no MHR model definition in {directory} or {nested}"
            ),
        }
    }
}
impl std::error::Error for MhrAssetDirectoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Inspect { source, .. } => Some(source),
            Self::Missing { .. } => None,
        }
    }
}
