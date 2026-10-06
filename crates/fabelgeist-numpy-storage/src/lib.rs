//! NumPy array storage for Burn.
//!
//! Reads `.npy` files and `.npz` archives (stored or deflated, zip64 included)
//! and hands the contents back as ordered bespoke value collections or Burn tensors.
//! Pure Rust: no NumPy, no C zlib.
//!
//! ```no_run
//! # fn main() -> anyhow::Result<()> {
//! use burn::tensor::Device;
//! use fabelgeist_numpy_storage::Npz;
//!
//! let archive = Npz::open("weights.npz")?;
//! let weights = archive.array("layer0")?.to_tensor::<2>(&Device::default())?;
//! # let _ = weights;
//! # Ok(())
//! # }
//! ```

pub mod npy;
pub mod npz;
pub mod zip;

pub use npy::{
    Dtype, NpyArray, NpyByteState, NpyByteStates, NpyFloatValue, NpyFloatValues, NpyIntegerValue,
    NpyIntegerValues,
};
pub use npz::Npz;
pub use zip::ZipArchive;

/// Reads a standalone `.npy` file.
pub use npy::read as read_npy;
