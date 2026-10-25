//! NumPy array storage for Burn.
//!
//! Reads `.npy` files and `.npz` archives (stored or deflated, zip64 included)
//! and retains decoded values in bespoke collections until Burn tensor adapters.
//! Pure Rust: no NumPy, no C zlib.
//!
//! ```no_run
//! # fn main() -> anyhow::Result<()> {
//! use burn::tensor::Device;
//! use fabelgeist_numpy_storage::{Npz, NpzArrayName};
//!
//! let archive = Npz::open("weights.npz")?;
//! let weights = archive.array(&NpzArrayName::new("layer0"))?.to_tensor::<2>(&Device::default())?;
//! # let _ = weights;
//! # Ok(())
//! # }
//! ```

pub use npy::{
    Dtype, NpyArray, NpyByteState, NpyByteStates, NpyDimension, NpyElementCount, NpyElementWidth,
    NpyFloat32Value, NpyFloat32Values, NpyIntegerValue, NpyIntegerValues, NpyLayoutError,
    NpyOccupancy, NpyPayloadLength, NpyRank, NpyShape, NpyTensorLayout,
};
pub use npz::{Npz, NpzArrayName};
pub use zip::{
    ArchiveMemberName, ArchiveMemberPresence, DeclaredUncompressedByteLength, ZipArchive,
};

pub mod npy;
pub mod npz;
pub mod zip;
