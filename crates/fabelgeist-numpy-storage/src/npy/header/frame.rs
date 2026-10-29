//! Native NPY framing admits the dictionary and payload into one header.

use anyhow::{Context, Result, bail};

use super::NpyHeader;

const MAGIC: &[u8] = b"\x93NUMPY";
const VERSION_START: usize = MAGIC.len();
const LENGTH_START: usize = VERSION_START + 2;
const SHORT_HEADER_START: usize = LENGTH_START + size_of::<u16>();
const LONG_HEADER_START: usize = LENGTH_START + size_of::<u32>();

enum HeaderVersion {
    ShortLength,
    LongLength,
}

impl<'a> NpyHeader<'a> {
    pub(in super::super) fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        let prefix = bytes
            .get(..SHORT_HEADER_START)
            .filter(|prefix| prefix.starts_with(MAGIC))
            .context("not a .npy array")?;
        let version = match prefix.get(VERSION_START..LENGTH_START) {
            Some([1, 0]) => HeaderVersion::ShortLength,
            Some([2 | 3, 0]) => HeaderVersion::LongLength,
            Some(version) => bail!("unsupported .npy version {version:?}"),
            None => bail!("not a .npy array"),
        };
        let start = version.header_start();
        let length = version.header_length(bytes)?;
        let (dictionary, payload) = bytes
            .get(start..)
            .and_then(|tail| tail.split_at_checked(length))
            .context("truncated .npy header")?;
        Self::from_dictionary(
            std::str::from_utf8(dictionary).context("non-UTF-8 .npy header")?,
            payload,
        )
    }
}

impl HeaderVersion {
    fn header_start(&self) -> usize {
        match self {
            Self::ShortLength => SHORT_HEADER_START,
            Self::LongLength => LONG_HEADER_START,
        }
    }

    fn header_length(&self, bytes: &[u8]) -> Result<usize> {
        match self {
            Self::ShortLength => {
                let encoded = bytes
                    .get(LENGTH_START..SHORT_HEADER_START)
                    .context("truncated .npy header")?
                    .try_into()?;
                Ok(usize::from(u16::from_le_bytes(encoded)))
            }
            Self::LongLength => {
                let encoded = bytes
                    .get(LENGTH_START..LONG_HEADER_START)
                    .context("truncated .npy header")?
                    .try_into()?;
                usize::try_from(u32::from_le_bytes(encoded))
                    .context(".npy header length does not fit this platform")
            }
        }
    }
}
