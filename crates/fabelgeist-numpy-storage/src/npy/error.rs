//! Errors owned by NumPy array decoding and standalone file reads.

use super::NpyLayoutError;
use derive_more::Display;
use std::{
    error::Error as StdError,
    fmt, io,
    num::ParseIntError,
    path::{Path, PathBuf},
    str::Utf8Error,
};
use thiserror::Error;

/// Result for admitting a serialized array.
pub type DecodeResult<T> = std::result::Result<T, NpyDecodeError>;
/// Result for reading and decoding a standalone file.
pub type ReadResult<T> = std::result::Result<T, NpyReadError>;

/// A required field in the serialized header dictionary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum NpyHeaderField {
    #[display("'descr'")]
    Dtype,
    #[display("'shape'")]
    Shape,
}

/// A serialized array rejection with its original native input or cause.
#[derive(Debug, Error)]
pub enum NpyDecodeError {
    #[error("not a .npy array")]
    Magic,
    #[error("truncated .npy header")]
    TruncatedHeader,
    #[error(fmt = fmt_header_encoding)]
    HeaderEncoding { source: Utf8Error },
    #[error("missing {0} in .npy header")]
    MissingField(NpyHeaderField),
    #[error("unsupported NumPy dtype {native_descr:?}")]
    Dtype {
        /// Rejected native descriptor after the original quote trimming.
        native_descr: String,
    },
    #[error("Fortran-ordered .npy arrays are not supported")]
    FortranOrder,
    #[error(fmt = fmt_shape_dimension)]
    ShapeDimension {
        /// Rejected serialized token after the original whitespace trimming.
        native_token: String,
        source: ParseIntError,
    },
    #[error(transparent)]
    Layout(#[from] NpyLayoutError),
}

/// A file read or decoder rejection with the original operating-system path.
#[derive(Debug, Error)]
pub enum NpyReadError {
    #[error(fmt = fmt_file_read)]
    Read {
        native_path: PathBuf,
        source: io::Error,
    },
    #[error(fmt = fmt_file_decode)]
    Decode {
        native_path: PathBuf,
        source: NpyDecodeError,
    },
}

fn fmt_header_encoding(source: &Utf8Error, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("non-UTF-8 .npy header")?;
    fmt_causes(source, formatter)
}

fn fmt_shape_dimension(
    _native_token: &str,
    source: &ParseIntError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    formatter.write_str("bad .npy shape")?;
    fmt_causes(source, formatter)
}

fn fmt_file_read(
    native_path: &Path,
    source: &io::Error,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(formatter, "reading {}", native_path.display())?;
    fmt_causes(source, formatter)
}

fn fmt_file_decode(
    native_path: &Path,
    source: &NpyDecodeError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(formatter, "parsing {}", native_path.display())?;
    fmt_causes(source, formatter)
}

fn fmt_causes(
    source: &(dyn StdError + 'static),
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if formatter.alternate() {
        let mut cause = Some(source);
        while let Some(error) = cause {
            write!(formatter, ": {error}")?;
            cause = error.source();
        }
    }
    Ok(())
}
