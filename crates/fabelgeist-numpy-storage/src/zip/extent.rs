//! ZIP byte quantities and archive-origin addressing.
//!
//! Declarations preserve the full wire width, including zero. Native indexing
//! and allocation ports retain the reader's casts and unchecked arithmetic.

use std::fmt;

/// Byte length of a member's encoded payload, before decompression.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CompressedByteLength(u64);

/// Declared decoded byte length of a ZIP member, including any NPY header.
///
/// This is archive metadata, not a verified decoded size or allocation limit.
/// Both ZIP and NPZ size queries return this bespoke type.
///
/// ```
/// use fabelgeist_numpy_storage::{DeclaredUncompressedByteLength, ZipArchive};
/// fn declared(archive: &ZipArchive) -> Option<DeclaredUncompressedByteLength> {
///     archive.uncompressed_size("weights.npy")
/// }
/// ```
///
/// Raw integer size returns are deliberately rejected:
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::ZipArchive;
/// fn declared(archive: &ZipArchive) -> Option<u64> {
///     archive.uncompressed_size("weights.npy")
/// }
/// ```
///
/// Compressed lengths cannot stand in for decoded declarations:
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{CompressedByteLength, DeclaredUncompressedByteLength};
/// let decoded: DeclaredUncompressedByteLength = CompressedByteLength::from_bytes(3);
/// ```
///
/// Archive positions are a separate quantity:
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{ArchiveByteOffset, DeclaredUncompressedByteLength};
/// let decoded: DeclaredUncompressedByteLength = ArchiveByteOffset::from_archive_start(3);
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DeclaredUncompressedByteLength(u64);

/// Byte position relative to the archive's first byte, whose origin is zero.
///
/// Local headers, the central directory and ZIP64 locator records share this
/// frame. A position does not certify that bytes exist at that location.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ArchiveByteOffset(u64);

impl CompressedByteLength {
    /// Admits a literal or decoded wire quantity without narrowing its width.
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    pub(super) fn is_saturated_ordinary(self) -> bool {
        self.0 == u64::from(u32::MAX)
    }

    /// Native slice addressing retains the inherited lossy platform cast.
    pub(super) fn native_slice_length(self) -> usize {
        self.0 as usize
    }
}

impl DeclaredUncompressedByteLength {
    /// Admits a literal or decoded wire declaration, including zero and u64::MAX.
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Projects the declaration into a native byte quantity for wire consumers.
    ///
    /// This does not measure or verify the decoded payload.
    pub const fn bytes(self) -> u64 {
        self.0
    }

    pub(super) fn is_saturated_ordinary(self) -> bool {
        self.0 == u64::from(u32::MAX)
    }

    /// Native Vec reservation retains the inherited lossy cast and policy.
    pub(super) fn native_allocation_capacity(self) -> usize {
        self.0 as usize
    }
}

impl ArchiveByteOffset {
    /// Admits a decoded or authored position in the archive-origin byte frame.
    pub const fn from_archive_start(bytes: u64) -> Self {
        Self(bytes)
    }

    pub(super) fn is_saturated_ordinary(self) -> bool {
        self.0 == u64::from(u32::MAX)
    }

    /// Native slice addressing retains the inherited lossy platform cast.
    pub(super) fn native_byte_index(self) -> usize {
        self.0 as usize
    }

    /// Advances over native wire framing, preserving unchecked u64 addition.
    ///
    /// Callers supply the fixed header plus native name/extra/comment widths.
    pub(super) fn advance_native_bytes(&mut self, bytes: usize) {
        self.0 += bytes as u64;
    }
}

impl fmt::Debug for CompressedByteLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for CompressedByteLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl fmt::Debug for DeclaredUncompressedByteLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for DeclaredUncompressedByteLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl fmt::Debug for ArchiveByteOffset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for ArchiveByteOffset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}
