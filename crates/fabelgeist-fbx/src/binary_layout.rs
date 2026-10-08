//! Native version admission and binary node-header layout selection.

use derive_more::{From, Into};

pub(super) const BINARY_SIGNATURE: &[u8] = b"Kaydara FBX Binary  \0\x1a\0";
// The existing reader checks only this prefix; the two marker bytes stay
// ignored.
pub(super) const BINARY_MAGIC_PREFIX: &[u8] = b"Kaydara FBX Binary  \0";
pub(super) const FILE_VERSION_OFFSET: usize = BINARY_SIGNATURE.len();
pub(super) const FILE_HEADER_BYTES: usize = FILE_VERSION_OFFSET + size_of::<u32>();

/// Open identity from the four-byte wire version; native projection serves
/// encoding.
/// Layout selection does not establish support for a version's other features.
#[derive(Clone, Copy, Debug, From, Into, PartialEq, Eq)]
pub(super) struct FbxVersion(u32);

/// The three end-offset/count/property-length words and one name-length byte.
#[derive(Clone, Copy)]
pub(super) enum HeaderFormat {
    Narrow,
    Wide,
}

impl FbxVersion {
    // Blender's FBX reader switches its element metadata to 64 bits at version
    // 7500.
    // See the format source linked by HeaderFormat::record_width.
    const WIDE_HEADER_VERSION: Self = Self(7500);

    pub(super) fn format(self) -> HeaderFormat {
        if self.0 >= Self::WIDE_HEADER_VERSION.0 {
            HeaderFormat::Wide
        } else {
            HeaderFormat::Narrow
        }
    }
}

impl HeaderFormat {
    /// Width of three 32/64-bit words plus a name-length byte, including null
    /// records.
    ///
    /// Blender's reader uses `<IIIB` (13 bytes) and `<QQQB` (25 bytes):
    /// <https://github.com/blender/blender-addons/blob/main/io_scene_fbx/parse_fbx.py>.
    pub(super) fn record_width(self) -> usize {
        match self {
            Self::Narrow => size_of::<[u32; 3]>() + size_of::<u8>(),
            Self::Wide => size_of::<[u64; 3]>() + size_of::<u8>(),
        }
    }
}
