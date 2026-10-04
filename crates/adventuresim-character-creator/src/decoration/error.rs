//! Decoration validation and library persistence failures.

use std::path::PathBuf;

use fabelgeist_armor::{material::MetalError, trim::TrimFinishError};

use super::LibraryName;

#[derive(Debug, thiserror::Error)]
pub enum DecorationError {
    #[error("{0}")]
    Engraving(#[from] MetalError),
    #[error("{0}")]
    Trim(#[from] TrimFinishError),
}

#[derive(Debug, thiserror::Error)]
pub enum DecorationLibraryError {
    #[error("{name} decorates nothing")]
    Plain { name: LibraryName },
    #[error("invalid decoration {name}: {source}")]
    Invalid {
        name: LibraryName,
        #[source]
        source: Box<DecorationError>,
    },
    #[error("reading decoration library {path}: {source}", path = .path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing decoration library {path}: {source}", path = .path.display())]
    Decode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("creating decoration library directory {path}: {source}", path = .path.display())]
    Directory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("encoding decoration library: {0}")]
    Encode(#[source] serde_json::Error),
    #[error("saving decoration library {path}: {source}", path = .path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
