//! The supported scalar NPY dictionary, admitted before payload decoding.

use anyhow::{Context, Result, bail, ensure};

use super::{Dtype, NpyDimension, NpyShape};
use frame::HeaderFrame;

mod frame;

pub(super) struct NpyHeader<'a> {
    pub(super) dtype: Dtype,
    pub(super) shape: NpyShape,
    /// Serialized element bytes borrowed until the admitted array takes ownership.
    pub(super) payload: &'a [u8],
}

struct HeaderDictionary {
    dtype: Dtype,
    shape: NpyShape,
}

enum HeaderField {
    Descriptor,
    Order,
    Shape,
}

enum StorageOrder {
    RowMajor,
    Fortran,
}

impl<'a> NpyHeader<'a> {
    pub(super) fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        let frame = HeaderFrame::from_bytes(bytes)?;
        let header = &bytes[frame.dictionary.clone()];
        let dictionary =
            HeaderDictionary::parse(std::str::from_utf8(header).context("non-UTF-8 .npy header")?)?;
        let length = dictionary
            .shape
            .element_count()
            .payload_length(dictionary.dtype)?;
        let payload = length.admit(&bytes[frame.dictionary.end..])?;
        Ok(Self {
            dtype: dictionary.dtype,
            shape: dictionary.shape,
            payload,
        })
    }
}

impl HeaderDictionary {
    fn parse(header: &str) -> Result<Self> {
        let mut cursor = header;
        take(&mut cursor, "{")?;
        // These remain lexical fields only until every required key is present.
        // Dtype/order rejection deliberately precedes parsing shape products.
        let mut descriptor = None;
        let mut order = None;
        let mut dimensions = None;
        loop {
            cursor = cursor.trim_start();
            if let Some(rest) = cursor.strip_prefix('}') {
                ensure!(rest.trim().is_empty(), "trailing .npy header text");
                break;
            }
            let key = quoted(&mut cursor)?;
            take(&mut cursor, ":")?;
            let field = HeaderField::parse(key)?;
            match field {
                HeaderField::Descriptor => ensure!(
                    descriptor.replace(quoted(&mut cursor)?).is_none(),
                    "duplicate 'descr' in .npy header"
                ),
                HeaderField::Order => ensure!(
                    order.replace(StorageOrder::parse(&mut cursor)?).is_none(),
                    "duplicate 'fortran_order' in .npy header"
                ),
                HeaderField::Shape => {
                    take(&mut cursor, "(")?;
                    let end = cursor.find(')').context("bad .npy shape")?;
                    ensure!(
                        dimensions.replace(&cursor[..end]).is_none(),
                        "duplicate 'shape' in .npy header"
                    );
                    cursor = &cursor[end + 1..];
                }
            }
            cursor = cursor.trim_start();
            if let Some(rest) = cursor.strip_prefix(',') {
                cursor = rest;
            } else {
                ensure!(cursor.starts_with('}'), "missing .npy header separator");
            }
        }
        let descriptor = descriptor.context("missing 'descr' in .npy header")?;
        let order = order.context("missing 'fortran_order' in .npy header")?;
        let dimensions = dimensions.context("missing 'shape' in .npy header")?;
        let dtype = Dtype::parse(descriptor)?;
        if let StorageOrder::Fortran = order {
            bail!("Fortran-ordered .npy arrays are not supported");
        }
        Ok(Self {
            dtype,
            shape: NpyShape::from_header_tuple(dimensions)?,
        })
    }
}

impl HeaderField {
    fn parse(key: &str) -> Result<Self> {
        match key {
            "descr" => Ok(Self::Descriptor),
            "fortran_order" => Ok(Self::Order),
            "shape" => Ok(Self::Shape),
            _ => bail!("unsupported .npy header field {key:?}"),
        }
    }
}

impl StorageOrder {
    fn parse(cursor: &mut &str) -> Result<Self> {
        let text = cursor.trim_start();
        if let Some(rest) = text.strip_prefix("True") {
            *cursor = rest;
            Ok(Self::Fortran)
        } else if let Some(rest) = text.strip_prefix("False") {
            *cursor = rest;
            Ok(Self::RowMajor)
        } else {
            bail!("bad 'fortran_order' in .npy header")
        }
    }
}

impl NpyShape {
    fn from_header_tuple(text: &str) -> Result<Self> {
        let text = text.trim();
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
        Ok(Self::try_from(dimensions)?)
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
