//! Exact WGSL storage format admission through the shared color vocabulary.

use super::TextureFormat;
use fabelgeist_color::PixelFormat;
use std::{error::Error, fmt};

pub type StorageFormatResult<T> = Result<T, StorageFormatError>;

/// This native format has no WGSL storage texture spelling.
#[derive(Clone, Copy, Debug)]
pub struct StorageFormatError {
    format: TextureFormat,
}

impl TextureFormat {
    /// Name this exact storage format, without changing its channel encoding.
    /// Device features and resource usage are checked by native validation.
    pub fn to_wgsl_storage_format(&self) -> StorageFormatResult<&'static str> {
        PixelFormat::from(*self)
            .to_wgsl_storage_format()
            .ok_or(StorageFormatError { format: *self })
    }
}

impl fmt::Display for StorageFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "texture format {:?} has no WGSL storage format",
            self.format
        )
    }
}

impl Error for StorageFormatError {}
