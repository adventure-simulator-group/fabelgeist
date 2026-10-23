//! Bounded NPY framing and the supported scalar header dictionary.

use anyhow::{Context, Result, bail, ensure};

use super::{Dtype, NpyDimension, NpyShape};

const MAGIC: &[u8] = b"\x93NUMPY";

pub(super) struct NpyHeader<'a> {
    pub(super) dtype: Dtype,
    pub(super) shape: NpyShape,
    pub(super) payload: &'a [u8],
}

/// Borrowed wire fields; no array metadata is committed until all are present.
struct HeaderDictionary<'a> {
    descriptor: &'a str,
    fortran_order: bool,
    dimensions: &'a str,
}

impl<'a> NpyHeader<'a> {
    pub(super) fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        if bytes.len() < 10 || !bytes.starts_with(MAGIC) {
            bail!("not a .npy array");
        }
        // Versioned field widths and positions are native format framing.
        let (header_start, header_length) = match &bytes[6..8] {
            [1, 0] => {
                let encoded = bytes[8..10].try_into()?;
                (10usize, usize::from(u16::from_le_bytes(encoded)))
            }
            [2 | 3, 0] => {
                let encoded = bytes
                    .get(8..12)
                    .context("truncated .npy header")?
                    .try_into()?;
                let length = usize::try_from(u32::from_le_bytes(encoded))
                    .context(".npy header length does not fit this platform")?;
                (12usize, length)
            }
            version => bail!("unsupported .npy version {version:?}"),
        };
        let header_end = header_start
            .checked_add(header_length)
            .context("truncated .npy header")?;
        let header = bytes
            .get(header_start..header_end)
            .context("truncated .npy header")?;
        let dictionary =
            HeaderDictionary::parse(std::str::from_utf8(header).context("non-UTF-8 .npy header")?)?;
        let dtype = Dtype::parse(dictionary.descriptor)?;
        if dictionary.fortran_order {
            bail!("Fortran-ordered .npy arrays are not supported");
        }
        let shape = dictionary.shape()?;
        let length = shape.element_count().payload_length(dtype)?;
        let payload = length.admit(&bytes[header_end..])?;
        Ok(Self {
            dtype,
            shape,
            payload,
        })
    }
}

impl<'a> HeaderDictionary<'a> {
    fn parse(header: &'a str) -> Result<Self> {
        let mut cursor = header;
        take(&mut cursor, "{")?;
        let mut descriptor = None;
        let mut fortran_order = None;
        let mut dimensions = None;
        loop {
            cursor = cursor.trim_start();
            if let Some(rest) = cursor.strip_prefix('}') {
                ensure!(rest.trim().is_empty(), "trailing .npy header text");
                break;
            }
            let key = quoted(&mut cursor)?;
            take(&mut cursor, ":")?;
            match key {
                "descr" => ensure!(
                    descriptor.replace(quoted(&mut cursor)?).is_none(),
                    "duplicate 'descr' in .npy header"
                ),
                "fortran_order" => ensure!(
                    fortran_order.replace(boolean(&mut cursor)?).is_none(),
                    "duplicate 'fortran_order' in .npy header"
                ),
                "shape" => {
                    take(&mut cursor, "(")?;
                    let end = cursor.find(')').context("bad .npy shape")?;
                    ensure!(
                        dimensions.replace(&cursor[..end]).is_none(),
                        "duplicate 'shape' in .npy header"
                    );
                    cursor = &cursor[end + 1..];
                }
                _ => bail!("unsupported .npy header field {key:?}"),
            }
            cursor = cursor.trim_start();
            if let Some(rest) = cursor.strip_prefix(',') {
                cursor = rest;
            } else {
                ensure!(cursor.starts_with('}'), "missing .npy header separator");
            }
        }
        Ok(Self {
            descriptor: descriptor.context("missing 'descr' in .npy header")?,
            fortran_order: fortran_order.context("missing 'fortran_order' in .npy header")?,
            dimensions: dimensions.context("missing 'shape' in .npy header")?,
        })
    }

    fn shape(&self) -> Result<NpyShape> {
        let text = self.dimensions.trim();
        let dimensions = if text.is_empty() {
            Vec::new()
        } else {
            ensure!(text.contains(','), "bad .npy shape: expected a tuple");
            text.strip_suffix(',')
                .unwrap_or(text)
                .split(',')
                .map(|token| {
                    token
                        .trim()
                        .parse::<usize>()
                        .map(NpyDimension::from)
                        .context("bad .npy shape")
                })
                .collect::<Result<_>>()?
        };
        Ok(NpyShape::try_from(dimensions)?)
    }
}

fn take(cursor: &mut &str, token: &str) -> Result<()> {
    *cursor = cursor
        .trim_start()
        .strip_prefix(token)
        .with_context(|| format!("bad .npy header: expected {token:?}"))?;
    Ok(())
}

fn quoted<'a>(cursor: &mut &'a str) -> Result<&'a str> {
    let text = cursor.trim_start();
    let quote = text.chars().next().context("truncated .npy string")?;
    ensure!(matches!(quote, '\'' | '"'), "expected a .npy string");
    let end = text[1..].find(quote).context("unterminated .npy string")?;
    let value = &text[1..end + 1];
    ensure!(
        !value.contains('\\'),
        "escaped .npy strings are not supported"
    );
    *cursor = &text[end + 2..];
    Ok(value)
}

fn boolean(cursor: &mut &str) -> Result<bool> {
    let text = cursor.trim_start();
    if let Some(rest) = text.strip_prefix("True") {
        *cursor = rest;
        Ok(true)
    } else if let Some(rest) = text.strip_prefix("False") {
        *cursor = rest;
        Ok(false)
    } else {
        bail!("bad 'fortran_order' in .npy header")
    }
}
