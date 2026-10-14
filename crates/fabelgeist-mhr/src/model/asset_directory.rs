//! Native asset-directory selection and its missing-definition rejection.

use std::path::{Path, PathBuf};

use super::{MODEL_DEFINITION, Mhr};

type DirectoryResult<T> = std::result::Result<T, MhrAssetDirectoryError>;

/// The native loader found no definition at either of its two lookup paths.
///
/// Metadata errors retain the existing `Path::is_file` false policy, so this
/// classification does not claim that both paths were successfully inspected.
#[derive(Debug)]
pub enum MhrAssetDirectoryError {
    Missing {
        /// Rejected OS path provenance, without normalization or text conversion.
        native_directory: PathBuf,
    },
}

impl std::fmt::Display for MhrAssetDirectoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { native_directory } => write!(
                formatter,
                "no MHR assets in {}: expected {MODEL_DEFINITION} there or under assets/",
                native_directory.display()
            ),
        }
    }
}

impl std::error::Error for MhrAssetDirectoryError {}

impl Mhr {
    // Native filesystem port: the caller's SDK Path and selected PathBuf feed
    // joins/reads directly. This selects spelling; it admits no resource type.
    // is_file intentionally keeps metadata failures indistinguishable from absence.
    pub(super) fn resolve_asset_directory(native_directory: &Path) -> DirectoryResult<PathBuf> {
        if native_directory.join(MODEL_DEFINITION).is_file() {
            return Ok(native_directory.to_path_buf());
        }
        let nested = native_directory.join("assets");
        if nested.join(MODEL_DEFINITION).is_file() {
            return Ok(nested);
        }
        Err(MhrAssetDirectoryError::Missing {
            native_directory: native_directory.to_path_buf(),
        })
    }
}
