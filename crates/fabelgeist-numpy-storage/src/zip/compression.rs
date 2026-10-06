//! Method codes are admitted once from central-directory wire metadata.

use std::fmt;

/// The archive reader's supported methods and a retained foreign method code.
/// Unknown methods remain queryable metadata; member decoding rejects them only
/// after checking the local header and payload bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZipCompression {
    Stored,
    Deflated,
    Unsupported(ZipCompressionCode),
}

/// An unsupported two-byte ZIP wire method, retained for its diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ZipCompressionCode(u16);

impl From<u16> for ZipCompression {
    fn from(code: u16) -> Self {
        match code {
            0 => Self::Stored,
            8 => Self::Deflated,
            code => Self::Unsupported(ZipCompressionCode(code)),
        }
    }
}

impl fmt::Display for ZipCompressionCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}
