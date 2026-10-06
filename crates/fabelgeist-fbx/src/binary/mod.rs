//! Native byte decoding and failure admission for binary FBX.
//!
//! Cursor positions, wire counts/encoding words and scalar byte arities remain
//! native representations here, directly at slice, allocation, range and
//! little-endian conversion ports. They are not forwarded as admitted metadata
//! or shared storage values. Framing and metadata ownership are separate work.
use crate::{Node, Prop};
use flate2::read::ZlibDecoder;
use std::io::Read;
mod error;
pub use error::{FbxDecodeError, Result};

struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    version: u32,
}
impl<'a> Reader<'a> {
    fn from_binary(data: &'a [u8]) -> Result<Self> {
        const MAGIC: &[u8] = b"Kaydara FBX Binary  \x00";
        if data.len() < 27 || &data[..MAGIC.len()] != MAGIC {
            if data.starts_with(b"; FBX") || data.starts_with(b"\xef\xbb\xbf; FBX") {
                return Err(FbxDecodeError::AsciiInput);
            }
            return Err(FbxDecodeError::NotBinaryInput);
        }
        let mut version_bytes = [0; 4];
        version_bytes.copy_from_slice(&data[23..27]);
        Ok(Self {
            data,
            position: 27,
            version: u32::from_le_bytes(version_bytes),
        })
    }
    fn take(&mut self, native_length: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(native_length)
            .ok_or(FbxDecodeError::ReadOverflow {
                native_offset: self.position,
                native_length,
            })?;
        if end > self.data.len() {
            return Err(FbxDecodeError::TruncatedRead {
                native_offset: self.position,
                native_length,
            });
        }
        let slice = &self.data[self.position..end];
        self.position = end;
        Ok(slice)
    }
    // take(N) guarantees the exact native array shape for from_le_bytes.
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
        if self.version >= 7500 {
            self.u64()
        } else {
            Ok(self.u32()? as u64)
        }
    }
    fn header_size(&self) -> usize {
        // Three native header words plus the one-byte name length.
        if self.version >= 7500 { 25 } else { 13 }
    }
    // The wire header and decoded slices meet their native conversion in this
    // port; counts and arity do not cross another semantic helper boundary.
    fn array<const N: usize, T: Copy>(&mut self, decode: impl Fn([u8; N]) -> T) -> Result<Vec<T>> {
        let count = self.u32()? as usize;
        let encoding = self.u32()?;
        let compressed_length = self.u32()? as usize;
        let raw = self.take(compressed_length)?;
        let bytes = if encoding == 0 {
            raw.to_vec()
        } else {
            let mut output = Vec::with_capacity(count * N);
            ZlibDecoder::new(raw)
                .read_to_end(&mut output)
                .map_err(|source| FbxDecodeError::Inflate { source })?;
            output
        };
        if bytes.len() < count * N {
            return Err(FbxDecodeError::ArrayShort {
                native_bytes: bytes.len(),
                native_count: count,
                native_width: N,
            });
        }
        Ok((0..count)
            .map(|index| {
                let mut scalar = [0; N];
                scalar.copy_from_slice(&bytes[index * N..(index + 1) * N]);
                decode(scalar)
            })
            .collect())
    }
}
impl Prop {
    fn from_reader(reader: &mut Reader<'_>) -> Result<Self> {
        let native_tag = reader.u8()?;
        Ok(match native_tag {
            b'Y' => Self::I16(i16::from_le_bytes(reader.take_array()?)),
            b'C' => Self::Bool(reader.u8()? != 0),
            b'I' => Self::I32(i32::from_le_bytes(reader.take_array()?)),
            b'F' => Self::F32(f32::from_le_bytes(reader.take_array()?)),
            b'D' => Self::F64(f64::from_le_bytes(reader.take_array()?)),
            b'L' => Self::I64(i64::from_le_bytes(reader.take_array()?)),
            b'f' => Self::ArrF32(reader.array(f32::from_le_bytes)?),
            b'd' => Self::ArrF64(reader.array(f64::from_le_bytes)?),
            b'i' => Self::ArrI32(reader.array(i32::from_le_bytes)?),
            b'l' => Self::ArrI64(reader.array(i64::from_le_bytes)?),
            b'b' => Self::ArrBool(reader.array::<1, _>(|bytes| bytes[0])?),
            b'S' => {
                let length = reader.u32()? as usize;
                Self::Str(reader.take(length)?.to_vec())
            }
            b'R' => {
                let length = reader.u32()? as usize;
                Self::Raw(reader.take(length)?.to_vec())
            }
            _ => return Err(FbxDecodeError::PropertyTag { native_tag }),
        })
    }
}
impl Node {
    /// Admit one native node record; None is the list-terminating null record.
    fn from_reader(reader: &mut Reader<'_>) -> Result<Option<Self>> {
        let end_offset = reader.header_word()? as usize;
        let count = reader.header_word()? as usize;
        let _property_list_length = reader.header_word()?;
        let name_length = reader.u8()? as usize;
        let name = String::from_utf8_lossy(reader.take(name_length)?).into_owned();
        if end_offset == 0 {
            return Ok(None);
        }
        let mut props = Vec::with_capacity(count);
        for _ in 0..count {
            props.push(Prop::from_reader(reader)?);
        }
        let mut children = Vec::new();
        let sentinel = reader.header_size();
        while reader.position + sentinel <= end_offset {
            let Some(child) = Self::from_reader(reader)? else {
                break;
            };
            children.push(child);
        }
        reader.position = end_offset;
        Ok(Some(Self {
            name,
            props,
            children,
        }))
    }
}

/// Parses the native top-level node list of a binary FBX file.
///
/// Every nonzero array encoding uses zlib. Native version words choose the
/// inherited narrow/wide layout without unsupported-version rejection.
pub fn parse(data: &[u8]) -> Result<Vec<Node>> {
    let mut reader = Reader::from_binary(data)?;
    let mut roots = Vec::new();
    while reader.position + reader.header_size() <= data.len() {
        let Some(node) = Node::from_reader(&mut reader)? else {
            break;
        };
        roots.push(node);
    }
    Ok(roots)
}
#[cfg(test)]
mod tests;
