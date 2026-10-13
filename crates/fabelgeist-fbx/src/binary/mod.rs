//! Native byte decoding and rejection admission for binary FBX.

use crate::array_encoding::FbxArrayEncodingCode;
use crate::binary_layout::{
    BINARY_MAGIC_PREFIX, FILE_HEADER_BYTES, FILE_VERSION_OFFSET, FbxVersion, HeaderFormat,
};
use crate::property_count::FbxPropertyCount;
use crate::{FbxArrayCount, FbxArrayKind, FbxPropertyTag, Node, Prop};
use flate2::read::ZlibDecoder;
use std::io::Read;

pub use error::{FbxDecodeError, Result};

mod error;

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    version: FbxVersion,
}

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or(FbxDecodeError::ReadOverflow {
                native_offset: self.pos,
                native_length: len,
            })?;
        if end > self.data.len() {
            return Err(FbxDecodeError::TruncatedRead {
                native_offset: self.pos,
                native_length: len,
            });
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut bytes = [0; N];
        bytes.copy_from_slice(self.take(N)?);
        Ok(bytes)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take_array()?))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take_array()?))
    }

    /// Node headers went 32-bit -> 64-bit in FBX 7500.
    fn header_word(&mut self) -> Result<u64> {
        match self.version.format() {
            HeaderFormat::Wide => self.u64(),
            HeaderFormat::Narrow => Ok(self.u32()? as u64),
        }
    }

    fn header_size(&self) -> usize {
        self.version.format().record_width()
    }
}

impl Node {
    /// Reads one node record. Returns `None` for the null record that terminates a list.
    fn from_reader(reader: &mut Reader<'_>) -> Result<Option<Node>> {
        let end_offset = reader.header_word()? as usize;
        let num_props = FbxPropertyCount::from(reader.header_word()?);
        let _prop_list_len = reader.header_word()?;
        let name_len = reader.u8()? as usize;
        let name = String::from_utf8_lossy(reader.take(name_len)?).into_owned();

        if end_offset == 0 {
            return Ok(None);
        }

        let props = read_props(reader, num_props)?;

        let mut children = Vec::new();
        let sentinel = reader.header_size();
        while reader.pos + sentinel <= end_offset {
            match Self::from_reader(reader)? {
                Some(child) => children.push(child),
                None => break,
            }
        }
        reader.pos = end_offset;

        Ok(Some(Self {
            name,
            props,
            children,
        }))
    }
}

/// Parses the top-level node list of a binary FBX file.
pub fn parse(data: &[u8]) -> Result<Vec<Node>> {
    if data.len() < FILE_HEADER_BYTES || &data[..BINARY_MAGIC_PREFIX.len()] != BINARY_MAGIC_PREFIX {
        // ASCII FBX is a different format sharing the extension. Saying so is
        // more use than "not a binary FBX file", because the fix is a re-export.
        if data.starts_with(b"; FBX") || data.starts_with(b"\xef\xbb\xbf; FBX") {
            return Err(FbxDecodeError::AsciiInput);
        }
        return Err(FbxDecodeError::NotBinaryInput);
    }
    // The preceding header-length guard proves this native four-byte copy.
    let mut wire_version = [0; size_of::<u32>()];
    wire_version.copy_from_slice(&data[FILE_VERSION_OFFSET..FILE_HEADER_BYTES]);
    let version = FbxVersion::from(u32::from_le_bytes(wire_version));
    let mut reader = Reader {
        data,
        pos: FILE_HEADER_BYTES,
        version,
    };

    let mut roots = Vec::new();
    while reader.pos + reader.header_size() <= data.len() {
        match Node::from_reader(&mut reader)? {
            Some(node) => roots.push(node),
            None => break,
        }
    }
    Ok(roots)
}

fn decode_array<const N: usize, T: Copy>(
    raw: &[u8],
    count: FbxArrayCount,
    encoding: FbxArrayEncodingCode,
    parse: impl Fn([u8; N]) -> T,
    kind: FbxArrayKind,
) -> Result<Vec<T>> {
    let bytes = if encoding.is_uncompressed() {
        raw.to_vec()
    } else {
        let mut out = Vec::with_capacity(count.native_byte_len(kind));
        ZlibDecoder::new(raw)
            .read_to_end(&mut out)
            .map_err(|source| FbxDecodeError::Inflate { source })?;
        out
    };
    if bytes.len() < count.native_byte_len(kind) {
        return Err(FbxDecodeError::ArrayShort {
            native_bytes: bytes.len(),
            count,
            kind,
        });
    }
    let width = kind.native_width();
    Ok((0..count.native_len())
        .map(|i| {
            let mut scalar = [0; N];
            scalar.copy_from_slice(&bytes[i * width..(i + 1) * width]);
            parse(scalar)
        })
        .collect())
}

fn read_props(reader: &mut Reader<'_>, count: FbxPropertyCount) -> Result<Vec<Prop>> {
    let mut props = Vec::with_capacity(count.native_len());
    for _ in 0..count.native_len() {
        let kind = FbxPropertyTag::from(reader.u8()?);
        let prop = match kind {
            FbxPropertyTag::INTEGER16 => Prop::I16(i16::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::BOOLEAN => Prop::Bool(reader.u8()? != 0),
            FbxPropertyTag::INTEGER32 => Prop::I32(i32::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::FLOAT32 => Prop::F32(f32::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::FLOAT64 => Prop::F64(f64::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::INTEGER64 => Prop::I64(i64::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::ARRAY_FLOAT32
            | FbxPropertyTag::ARRAY_FLOAT64
            | FbxPropertyTag::ARRAY_INTEGER64
            | FbxPropertyTag::ARRAY_INTEGER32
            | FbxPropertyTag::ARRAY_BOOLEAN => {
                let count = FbxArrayCount::from(reader.u32()?);
                // FBX array metadata stores encoding as a little-endian u32.
                // Admit that wire word before passing it to array decoding.
                let encoding = FbxArrayEncodingCode::from(reader.u32()?);
                let compressed_len = reader.u32()? as usize;
                let raw = reader.take(compressed_len)?;
                match kind {
                    FbxPropertyTag::ARRAY_FLOAT32 => Prop::ArrF32(decode_array(
                        raw,
                        count,
                        encoding,
                        f32::from_le_bytes,
                        FbxArrayKind::Float32,
                    )?),
                    FbxPropertyTag::ARRAY_FLOAT64 => Prop::ArrF64(decode_array(
                        raw,
                        count,
                        encoding,
                        f64::from_le_bytes,
                        FbxArrayKind::Float64,
                    )?),
                    FbxPropertyTag::ARRAY_INTEGER32 => Prop::ArrI32(decode_array(
                        raw,
                        count,
                        encoding,
                        i32::from_le_bytes,
                        FbxArrayKind::Integer32,
                    )?),
                    FbxPropertyTag::ARRAY_INTEGER64 => Prop::ArrI64(decode_array(
                        raw,
                        count,
                        encoding,
                        i64::from_le_bytes,
                        FbxArrayKind::Integer64,
                    )?),
                    _ => Prop::ArrBool(decode_array::<1, _>(
                        raw,
                        count,
                        encoding,
                        |b| b[0],
                        FbxArrayKind::Boolean,
                    )?),
                }
            }
            FbxPropertyTag::STRING => {
                let len = reader.u32()? as usize;
                Prop::Str(reader.take(len)?.to_vec())
            }
            FbxPropertyTag::RAW_BYTES => {
                let len = reader.u32()? as usize;
                Prop::Raw(reader.take(len)?.to_vec())
            }
            tag => return Err(FbxDecodeError::PropertyTag { tag }),
        };
        props.push(prop);
    }
    Ok(props)
}
