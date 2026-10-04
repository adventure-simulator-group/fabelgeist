//! Bounded procedural texture transfer failures.
use super::MapChannel;
#[derive(Debug)]
pub enum BakeDecodeError {
    Json(serde_json::Error),
    MissingHeaderLength,
    HeaderLimit,
    TruncatedHeader,
    Metadata,
    Dimensions { channel: MapChannel, size: u32 },
    MipCount { channel: MapChannel, levels: u32 },
    DuplicateChannel(MapChannel),
    TruncatedPayload,
    TrailingData,
}
impl std::fmt::Display for BakeDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => write!(f, "invalid bake JSON: {error}"),
            Self::MissingHeaderLength => f.write_str("missing bake header length"),
            Self::HeaderLimit => f.write_str("bake header exceeds limit"),
            Self::TruncatedHeader => f.write_str("truncated bake header"),
            Self::Metadata => f.write_str("invalid bake metadata"),
            Self::Dimensions { channel, size } => write!(f, "invalid {channel:?} map size {size}"),
            Self::MipCount { channel, levels } => {
                write!(f, "invalid {channel:?} mip count {levels}")
            }
            Self::DuplicateChannel(channel) => write!(f, "duplicate map channel {channel:?}"),
            Self::TruncatedPayload => f.write_str("truncated map payload"),
            Self::TrailingData => f.write_str("unexpected trailing bake data"),
        }
    }
}
impl std::error::Error for BakeDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}
