//! Native version admission and binary node-header layout selection.

/// An open version identity admitted from the four-byte file-header field.
/// The layout decision preserves the reader's existing cutoff without asserting
/// that a version's other format features are supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FbxVersion(u32);

impl From<u32> for FbxVersion {
    fn from(version: u32) -> Self {
        Self(version)
    }
}

impl FbxVersion {
    const WIDE_HEADER_VERSION: Self = Self(7500);

    pub(super) fn format(self) -> HeaderFormat {
        if self.0 >= Self::WIDE_HEADER_VERSION.0 {
            HeaderFormat::Wide
        } else {
            HeaderFormat::Narrow
        }
    }
}

/// Binary node headers use three 32-bit or three 64-bit words, then a name length.
#[derive(Clone, Copy)]
pub(super) enum HeaderFormat {
    Narrow,
    Wide,
}

impl HeaderFormat {
    /// Native byte width used by the reader's cursor and null-record checks.
    pub(super) fn record_width(self) -> usize {
        match self {
            Self::Narrow => 13,
            Self::Wide => 25,
        }
    }
}
