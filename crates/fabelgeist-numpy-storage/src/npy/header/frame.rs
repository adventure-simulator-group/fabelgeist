//! Native NPY version framing and the checked serialized header span.

use std::ops::Range;

use anyhow::{Context, Result, bail};

const MAGIC: &[u8] = b"\x93NUMPY";
const VERSION_START: usize = MAGIC.len();
const LENGTH_START: usize = VERSION_START + 2;
const SHORT_HEADER_START: usize = LENGTH_START + size_of::<u16>();
const LONG_HEADER_START: usize = LENGTH_START + size_of::<u32>();

pub(super) struct HeaderFrame {
    pub(super) dictionary: Range<usize>,
}

enum HeaderVersion {
    ShortLength,
    LongLength,
}

impl HeaderFrame {
    pub(super) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < SHORT_HEADER_START || !bytes.starts_with(MAGIC) {
            bail!("not a .npy array");
        }
        let version = match &bytes[VERSION_START..LENGTH_START] {
            [1, 0] => HeaderVersion::ShortLength,
            [2 | 3, 0] => HeaderVersion::LongLength,
            version => bail!("unsupported .npy version {version:?}"),
        };
        let start = version.header_start();
        let length = version.header_length(bytes)?;
        let end = start.checked_add(length).context("truncated .npy header")?;
        bytes.get(start..end).context("truncated .npy header")?;
        Ok(Self {
            dictionary: start..end,
        })
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
                let encoded = bytes[LENGTH_START..SHORT_HEADER_START].try_into()?;
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
