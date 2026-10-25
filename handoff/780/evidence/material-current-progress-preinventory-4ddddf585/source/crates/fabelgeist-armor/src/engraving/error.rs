//! Engraving admission and source failures, with their original causes.

use super::ReliefSource;
use crate::ornament::OrnamentError;
use crate::material::MetalTextureSize;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum EngravingError {
    #[error("Invalid engraving parameters")]
    Parameters,
    #[error(transparent)]
    Ornament(#[from] OrnamentError),
    #[error("Engraving {}: {source}", .path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Engraving {origin}: {source}")]
    Decode {
        origin: ReliefSource,
        #[source]
        source: image::ImageError,
    },
    #[error("a procedural ornament cannot admit a decoded image")]
    ImageSource,
    #[error("relief image extent {width}x{height} exceeds native signed wrap addresses")]
    ImageExtent { width: u32, height: u32 },
    #[error("relief image {width}x{height} exceeds native word addresses")]
    ImageWords {
        width: u32,
        height: u32,
        #[source]
        source: std::num::TryFromIntError,
    },
    #[error("relief image {width}x{height} samples exceed signed addresses for {size:?}")]
    SampleAddress {
        width: u32,
        height: u32,
        size: MetalTextureSize,
    },
    #[error("the relief image does not match the engraving's relief")]
    ReliefMismatch,
}
