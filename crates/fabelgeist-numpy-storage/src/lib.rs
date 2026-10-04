//! NumPy array storage for Burn.
//!
//! Reads `.npy` files and `.npz` archives (stored or deflated, zip64 included)
//! and returns admitted arrays with nominal values or Burn tensors.
//! Pure Rust: no NumPy, no C zlib.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use burn::tensor::Device;
//! use fabelgeist_numpy_storage::{Npz, NpzArrayName};
//! use fabelgeist_fs::NativeFile;
//!
//! let file = NativeFile::from(std::path::PathBuf::from("weights.npz"));
//! let archive = Npz::open(&file)?;
//! let weights = archive.array(&NpzArrayName::from("layer0"))?.to_tensor::<2>(&Device::default())?;
//! # let _ = weights;
//! # Ok(())
//! # }
//! ```

pub mod npy;
pub mod npz;
pub mod zip;

pub use npy::{
    Dtype, NpyArray, NpyByteState, NpyByteStates, NpyDimension, NpyElementCount, NpyElementOrdinal,
    NpyElementWidth, NpyFloatValue, NpyFloatValues, NpyIntegerValue, NpyIntegerValues,
    NpyOccupancy, NpyShape,
};
pub use npz::{Npz, NpzArrayError};
pub use zip::ZipArchive;
pub use zip::{ArchiveMemberName, ArchiveMemberPresence, NpzArrayName, ZipReadError};

pub use npy::{NpyDecodeError, NpyReadError};
