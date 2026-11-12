//! Method codes are admitted once from central-directory wire metadata.

use derive_more::{Display, Into};

/// An unsupported two-byte ZIP wire method, retained for its diagnostic.
#[derive(Clone, Copy, Debug, Display, Into, PartialEq, Eq)]
pub struct ZipCompressionCode(u16);

/// The archive reader's supported methods and a retained foreign method code.
/// Unknown methods remain queryable metadata; member decoding rejects them only
/// after checking the local header and payload bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ZipCompression {
    Stored,
    Deflated,
    Unsupported(ZipCompressionCode),
}

impl From<rawzip::CompressionMethod> for ZipCompression {
    fn from(method: rawzip::CompressionMethod) -> Self {
        match method {
            rawzip::CompressionMethod::STORE => Self::Stored,
            rawzip::CompressionMethod::DEFLATE => Self::Deflated,
            method => Self::Unsupported(ZipCompressionCode(method.as_u16())),
        }
    }
}
