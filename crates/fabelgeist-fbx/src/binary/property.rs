use super::metadata::NodeHeader;
use super::{
    FbxArrayCount, FbxArrayEncodingCode, FbxArrayKind, FbxDecodeError, FbxPropertyTag, FbxSection,
    Reader,
};
use crate::Prop;
use fabelgeist_storage::{
    StorageByteLength as Length, StorageByteOffset as Offset, StorageByteSpan as Span, StorageView,
};
use std::io::Read;
impl Reader<'_> {
    pub(super) fn properties(&mut self, header: &NodeHeader) -> Result<Vec<Prop>, FbxDecodeError> {
        if header.count.minimum_bytes() > header.properties {
            return Err(FbxDecodeError::PropertyCount {
                count: header.count,
                available: header.properties,
            });
        }
        let start = self.position;
        let outer = self.limit;
        let block = self.take(FbxSection::PropertyPayload, header.properties)?;
        self.position = start;
        self.limit = start
            .advance(block.length())
            .expect("admitted property block");
        let mut props = Vec::new();
        for _ in 0..header.count.0 {
            props.push(Prop::from_reader(self)?);
        }
        let actual = self
            .position
            .distance_from(start)
            .expect("forward property cursor");
        if actual != header.properties {
            return Err(FbxDecodeError::PropertyLength {
                declared: header.properties,
                actual,
            });
        }
        self.limit = outer;
        Ok(props)
    }
}
impl Prop {
    fn from_scalar(
        reader: &mut Reader<'_>,
        kind: super::metadata::ScalarKind,
    ) -> Result<Self, FbxDecodeError> {
        use super::metadata::ScalarKind;
        let bytes = reader.take(FbxSection::Scalar, kind.width())?;
        Ok(match kind {
            ScalarKind::Integer16 => Self::I16(i16::from_le_bytes(
                bytes.as_ref().try_into().expect("admitted scalar"),
            )),
            ScalarKind::Boolean => Self::Bool(bytes.as_ref()[0] != 0),
            ScalarKind::Integer32 => Self::I32(i32::from_le_bytes(
                bytes.as_ref().try_into().expect("admitted scalar"),
            )),
            ScalarKind::Integer64 => Self::I64(i64::from_le_bytes(
                bytes.as_ref().try_into().expect("admitted scalar"),
            )),
            ScalarKind::Float32 => Self::F32(f32::from_le_bytes(
                bytes.as_ref().try_into().expect("admitted scalar"),
            )),
            ScalarKind::Float64 => Self::F64(f64::from_le_bytes(
                bytes.as_ref().try_into().expect("admitted scalar"),
            )),
        })
    }
    fn from_reader(reader: &mut Reader<'_>) -> Result<Self, FbxDecodeError> {
        let at = reader.position;
        let tag = FbxPropertyTag::from(
            reader
                .take(FbxSection::PropertyTag, Length::from(1u64))?
                .as_ref()[0],
        );
        if let Some(kind) = tag.scalar_kind() {
            return Self::from_scalar(reader, kind);
        }
        Ok(match tag.0 {
            b'S' | b'R' => {
                let length = reader
                    .take(FbxSection::PropertyPayload, Length::from(4u64))?
                    .decode_u32(Offset::default())
                    .expect("admitted length word");
                let bytes = reader
                    .take(FbxSection::PropertyPayload, Length::from(u64::from(length)))?
                    .as_ref()
                    .to_vec();
                if tag.0 == b'S' {
                    Self::Str(bytes)
                } else {
                    Self::Raw(bytes)
                }
            }
            b'f' | b'd' | b'i' | b'l' | b'b' => {
                let kind = match tag.0 {
                    b'f' => FbxArrayKind::Float32,
                    b'd' => FbxArrayKind::Float64,
                    b'i' => FbxArrayKind::Integer32,
                    b'l' => FbxArrayKind::Integer64,
                    _ => FbxArrayKind::Boolean,
                };
                EncodedArray::from_reader(reader, kind)?.decode()?
            }
            _ => return Err(FbxDecodeError::PropertyTag { tag, at }),
        })
    }
}
enum ArrayEncoding {
    Plain,
    Zlib,
}
struct EncodedArray<'a> {
    kind: FbxArrayKind,
    count: FbxArrayCount,
    encoding: ArrayEncoding,
    raw: StorageView<'a>,
}
impl<'a> EncodedArray<'a> {
    fn from_reader(reader: &mut Reader<'a>, kind: FbxArrayKind) -> Result<Self, FbxDecodeError> {
        let words = reader.take(FbxSection::ArrayMetadata, Length::from(12u64))?;
        let count = FbxArrayCount::from(
            words
                .decode_u32(Offset::default())
                .expect("admitted array metadata"),
        );
        let code = FbxArrayEncodingCode::from(
            words
                .decode_u32(Offset::from(4u64))
                .expect("admitted array metadata"),
        );
        let length = Length::from(u64::from(
            words
                .decode_u32(Offset::from(8u64))
                .expect("admitted array metadata"),
        ));
        let encoding = match code.0 {
            0 => ArrayEncoding::Plain,
            1 => ArrayEncoding::Zlib,
            _ => return Err(FbxDecodeError::ArrayEncoding(code)),
        };
        Ok(Self {
            kind,
            count,
            encoding,
            raw: reader.take(FbxSection::ArrayPayload, length)?,
        })
    }
    fn decode(self) -> Result<Prop, FbxDecodeError> {
        let expected = self.kind.payload_length(self.count);
        let owned;
        let bytes = match self.encoding {
            ArrayEncoding::Plain => self.raw,
            ArrayEncoding::Zlib => {
                let mut decoder = flate2::read::ZlibDecoder::new(self.raw.as_ref());
                let mut prefix = Vec::new();
                decoder
                    .by_ref()
                    .take(u64::from(expected))
                    .read_to_end(&mut prefix)
                    .map_err(|source: std::io::Error| -> FbxDecodeError {
                        FbxDecodeError::Inflate {
                            kind: self.kind,
                            count: self.count,
                            source,
                        }
                    })?;
                // Consume and validate the complete stream while retaining only the
                // declared elements; accepted trailing decoded bytes stay ignored.
                std::io::copy(&mut decoder, &mut std::io::sink()).map_err(
                    |source: std::io::Error| -> FbxDecodeError {
                        FbxDecodeError::Inflate {
                            kind: self.kind,
                            count: self.count,
                            source,
                        }
                    },
                )?;
                owned = prefix;
                StorageView::from(owned.as_slice())
            }
        };
        if bytes.length() < expected {
            return Err(FbxDecodeError::ArrayShort {
                kind: self.kind,
                count: self.count,
                actual: bytes.length(),
            });
        }
        let payload = bytes
            .portion(Span {
                offset: Offset::default(),
                length: expected,
            })
            .expect("admitted array prefix");
        Ok(match self.kind {
            FbxArrayKind::Float32 => {
                let mut values = Vec::new();
                for word in payload.as_ref().as_chunks::<4>().0 {
                    values.push(f32::from_le_bytes(*word));
                }
                Prop::ArrF32(values)
            }
            FbxArrayKind::Float64 => {
                let mut values = Vec::new();
                for word in payload.as_ref().as_chunks::<8>().0 {
                    values.push(f64::from_le_bytes(*word));
                }
                Prop::ArrF64(values)
            }
            FbxArrayKind::Integer32 => {
                let mut values = Vec::new();
                for word in payload.as_ref().as_chunks::<4>().0 {
                    values.push(i32::from_le_bytes(*word));
                }
                Prop::ArrI32(values)
            }
            FbxArrayKind::Integer64 => {
                let mut values = Vec::new();
                for word in payload.as_ref().as_chunks::<8>().0 {
                    values.push(i64::from_le_bytes(*word));
                }
                Prop::ArrI64(values)
            }
            FbxArrayKind::Boolean => Prop::ArrBool(payload.as_ref().to_vec()),
        })
    }
}
