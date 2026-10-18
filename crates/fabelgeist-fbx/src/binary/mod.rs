//! Native byte decoding and rejection admission for binary FBX.

use crate::array_encoding::FbxArrayEncoding;
use crate::array_layout::FbxArrayPayloadLength;
use crate::binary_layout::{
    BINARY_MAGIC_PREFIX, FILE_HEADER_BYTES, FILE_VERSION_OFFSET, FbxVersion, HeaderFormat,
};
use crate::property_count::FbxPropertyCount;
use crate::property_tag::FbxPropertyTag;
use crate::{FbxArrayCount, Node, Prop};
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

    fn array<T: Copy, const WIDTH: usize>(
        &mut self,
        parse: impl Fn([u8; WIDTH]) -> T,
    ) -> Result<Vec<T>> {
        let count = FbxArrayCount::from_le_bytes(self.take_array()?);
        let encoding = FbxArrayEncoding::try_from_le_bytes(self.take_array()?)?;
        let payload_length = FbxArrayPayloadLength::from_le_bytes(self.take_array()?);
        let raw = self.take(payload_length.native_len())?;
        decode_array(raw, count, encoding, parse)
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

fn decode_array<T: Copy, const WIDTH: usize>(
    raw: &[u8],
    count: FbxArrayCount,
    encoding: FbxArrayEncoding,
    parse: impl Fn([u8; WIDTH]) -> T,
) -> Result<Vec<T>> {
    let bytes = match encoding {
        FbxArrayEncoding::Uncompressed => raw.to_vec(),
        FbxArrayEncoding::Zlib => {
            let mut out = Vec::with_capacity(count.native_byte_len::<WIDTH>());
            ZlibDecoder::new(raw)
                .read_to_end(&mut out)
                .map_err(|source| FbxDecodeError::Inflate { source })?;
            out
        }
    };
    if bytes.len() < count.native_byte_len::<WIDTH>() {
        return Err(FbxDecodeError::ArrayShort {
            native_bytes: bytes.len(),
            count,
            native_width: WIDTH,
        });
    }
    Ok((0..count.native_len())
        .map(|i| {
            let mut element = [0; WIDTH];
            element.copy_from_slice(&bytes[i * WIDTH..(i + 1) * WIDTH]);
            parse(element)
        })
        .collect())
}

fn read_props(reader: &mut Reader<'_>, count: FbxPropertyCount) -> Result<Vec<Prop>> {
    let mut props = Vec::with_capacity(count.native_len());
    for _ in 0..count.native_len() {
        let kind = FbxPropertyTag::try_from(reader.u8()?).map_err(|error| {
            FbxDecodeError::PropertyTag {
                native_tag: error.input,
            }
        })?;
        let prop = match kind {
            FbxPropertyTag::Integer16 => Prop::I16(i16::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::Boolean => Prop::Bool(reader.u8()? != 0),
            FbxPropertyTag::Integer32 => Prop::I32(i32::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::Float32 => Prop::F32(f32::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::Float64 => Prop::F64(f64::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::Integer64 => Prop::I64(i64::from_le_bytes(reader.take_array()?)),
            FbxPropertyTag::ArrayFloat32 => Prop::ArrF32(reader.array(f32::from_le_bytes)?),
            FbxPropertyTag::ArrayFloat64 => Prop::ArrF64(reader.array(f64::from_le_bytes)?),
            FbxPropertyTag::ArrayInteger32 => Prop::ArrI32(reader.array(i32::from_le_bytes)?),
            FbxPropertyTag::ArrayInteger64 => Prop::ArrI64(reader.array(i64::from_le_bytes)?),
            FbxPropertyTag::ArrayBoolean => Prop::ArrBool(reader.array(u8::from_le_bytes)?),
            FbxPropertyTag::String => {
                let len = reader.u32()? as usize;
                Prop::Str(reader.take(len)?.to_vec())
            }
            FbxPropertyTag::RawBytes => {
                let len = reader.u32()? as usize;
                Prop::Raw(reader.take(len)?.to_vec())
            }
        };
        props.push(prop);
    }
    Ok(props)
}
