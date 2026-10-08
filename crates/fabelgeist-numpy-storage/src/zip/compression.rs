//! Method codes are admitted once from central-directory wire metadata.

use derive_more::Display;

/// An unsupported two-byte ZIP wire method, retained for its diagnostic.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub(super) struct ZipCompressionCode(u16);

/// The archive reader's supported methods and a retained foreign method code.
/// Unknown methods remain queryable metadata; member decoding rejects them only
/// after checking the local header and payload bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZipCompression {
    Stored,
    Deflated,
    Unsupported(ZipCompressionCode),
}

impl From<u16> for ZipCompression {
    fn from(code: u16) -> Self {
        // PKWARE APPNOTE 4.4.5 assigns stored=0 and deflated=8; see the guide.
        match code {
            0 => Self::Stored,
            8 => Self::Deflated,
            code => Self::Unsupported(ZipCompressionCode(code)),
        }
    }
}
