//! Structured failures for hair's external file formats and dataset admission.

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, HairDataError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HairDataFormat {
    Npy,
    Npz,
    Ply,
    Usc,
}

#[derive(Debug, thiserror::Error)]
pub enum HairDataError {
    #[error("hair data I/O failed")]
    Io(#[from] std::io::Error),
    #[error("reading hair data from {path:?}")]
    FileIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("reading hair data from {path:?}")]
    File {
        path: PathBuf,
        #[source]
        source: Box<HairDataError>,
    },
    #[error("hair data contains invalid UTF-8")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("hair data contains an invalid integer")]
    Integer(#[from] std::num::ParseIntError),
    #[error("hair data contains an invalid float")]
    Float(#[from] std::num::ParseFloatError),
    #[error("hair data has an invalid fixed-width field")]
    FieldWidth(#[from] std::array::TryFromSliceError),
    #[error("hair dataset metadata is invalid")]
    Json(#[from] serde_json::Error),
    #[error("missing or invalid {format:?} signature")]
    InvalidSignature { format: HairDataFormat },
    #[error("truncated {format:?} data")]
    Truncated { format: HairDataFormat },
    #[error("array shape {shape:?} does not match its {actual_values} values")]
    ShapeMismatch {
        shape: Vec<usize>,
        actual_values: usize,
    },
    #[error("ZIP entry {name} exceeds the supported archive width")]
    ZipEntryTooLarge { name: String },
    #[error("unsupported NumPy version {version}")]
    NpyVersion { version: u8 },
    #[error("missing or invalid NumPy header field {name}")]
    NpyField { name: String },
    #[error("Fortran-ordered NumPy arrays are unsupported")]
    FortranOrder,
    #[error("unsupported NumPy dtype {name}")]
    NpyDtype { name: String },
    #[error("invalid ZIP central directory")]
    ZipDirectory,
    #[error("invalid local ZIP header for {name}")]
    ZipLocalHeader { name: String },
    #[error("unsupported ZIP compression method {method} for {name}")]
    ZipCompression { name: String, method: u16 },
    #[error("deflating ZIP entry {name}: {details:?}")]
    Deflate {
        name: String,
        details: miniz_oxide::inflate::DecompressError,
    },
    #[error("reading archive entry {name}")]
    ArchiveEntry {
        name: String,
        #[source]
        source: Box<HairDataError>,
    },
    #[error("unsupported PLY scalar type {name}")]
    PlyScalar { name: String },
    #[error("unsupported PLY encoding {name}")]
    PlyEncoding { name: String },
    #[error("a PLY property appears before any element")]
    PlyPropertyWithoutElement,
    #[error("a PLY surface needs x, y and z vertex properties")]
    PlyMissingCoordinates,
    #[error("the PLY surface contains no faces")]
    PlyMissingFaces,
    #[error("a PLY face refers past the last vertex")]
    PlyVertexIndex,
    #[error("USC points per strand exceed the format width")]
    UscPointCount,
    #[error("dataset {path:?} has no array {name}")]
    MissingDatasetArray { path: PathBuf, name: String },
    #[error("dataset positions have shape {shape:?}; expected strands by points by three")]
    DatasetPositionShape { shape: Vec<usize> },
    #[error("dataset strands must have at least a root and tip")]
    DatasetPointCount,
}
