//! Engraving admission and source failures, with their original causes.

use super::ReliefSource;
use crate::ornament::OrnamentError;
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
    #[error("the relief image does not match the engraving's relief")]
    ReliefMismatch,
}
