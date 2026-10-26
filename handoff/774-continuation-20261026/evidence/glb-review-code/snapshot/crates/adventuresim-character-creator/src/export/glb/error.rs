//! Failures preparing or writing a binary glTF frame.

use std::{collections::TryReserveError, fmt, io, num::TryFromIntError, path::PathBuf};

/// The result of preparing or writing a complete GLB frame.
pub type GlbResult<T> = std::result::Result<T, GlbError>;

/// The native length field whose representation failed.
#[derive(Clone, Copy, Debug)]
pub enum GlbLengthRole {
    Json,
    Binary,
    Container,
}

/// A payload or complete frame cannot be represented by its native lengths.
#[derive(Debug, thiserror::Error)]
pub enum GlbLayoutError {
    #[error("GLB {role} byte length overflows the host address space")]
    Overflow { role: GlbLengthRole },
    #[error("GLB {role} byte length {bytes} exceeds the unsigned 32-bit format field")]
    Width {
        role: GlbLengthRole,
        bytes: usize,
        #[source]
        source: TryFromIntError,
    },
}

/// Concrete serialization, layout, allocation and filesystem failures.
#[derive(Debug, thiserror::Error)]
pub enum GlbError {
    #[error("serializing GLB JSON")]
    Serialize(#[from] serde_json::Error),
    #[error(transparent)]
    Layout(#[from] GlbLayoutError),
    #[error("reserving the complete GLB frame")]
    Reserve(#[from] TryReserveError),
    #[error("creating export directory {}", path.display())]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("writing {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl fmt::Display for GlbLengthRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Json => "JSON chunk",
            Self::Binary => "BIN chunk",
            Self::Container => "container",
        })
    }
}
