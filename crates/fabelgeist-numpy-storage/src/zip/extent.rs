//! Declared decoded ZIP sizes, distinct from verified payload lengths.

use derive_more::{Debug, Display};

/// Declared decoded byte length of a ZIP member, including any NPY header.
///
/// This is archive metadata, not a verified decoded size or allocation limit.
/// Both ZIP and NPZ size queries return this bespoke type.
///
/// ```
/// use fabelgeist_numpy_storage::{ArchiveMemberName, DeclaredUncompressedByteLength, ZipArchive};
/// fn declared(archive: &ZipArchive) -> Option<DeclaredUncompressedByteLength> {
///     archive.uncompressed_size(&ArchiveMemberName::from("weights.npy"))
/// }
/// ```
///
/// Raw integer size returns are deliberately rejected:
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{ArchiveMemberName, ZipArchive};
/// fn declared(archive: &ZipArchive) -> Option<u64> {
///     archive.uncompressed_size(&ArchiveMemberName::from("weights.npy"))
/// }
/// ```
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
#[debug("{_0:?}")]
pub struct DeclaredUncompressedByteLength(u64);

impl DeclaredUncompressedByteLength {
    /// Admits a decoded wire declaration, including zero and u64::MAX.
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Projects the unverified declaration at an external byte-count boundary.
    pub const fn bytes(self) -> u64 {
        self.0
    }

    /// Native Vec reservation retains the inherited lossy cast and policy.
    pub(super) fn native_allocation_capacity(self) -> usize {
        self.0 as usize
    }
}
