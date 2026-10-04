//! Checked binary FBX framing; scene interpretation remains in its owning layer.
mod error;
mod metadata;
mod property;
#[cfg(test)]
mod tests;
use crate::{FbxRecordName, Node};
pub use error::{FbxDecodeError, FbxFormatViolation};
use fabelgeist_storage::{
    StorageBoundsError, StorageByteLength as Length, StorageByteOffset as Offset,
    StorageByteSpan as Span, StorageView,
};
pub use metadata::{
    FbxArrayCount, FbxArrayEncodingCode, FbxArrayKind, FbxPropertyCount, FbxPropertyTag,
    FbxSection, FbxVersion,
};
use metadata::{HeaderFormat, NodeHeader};

struct Reader<'a> {
    data: StorageView<'a>,
    position: Offset,
    limit: Offset,
    format: HeaderFormat,
}
impl<'a> TryFrom<StorageView<'a>> for Reader<'a> {
    type Error = FbxDecodeError;
    fn try_from(data: StorageView<'a>) -> Result<Self, FbxDecodeError> {
        const MAGIC: &[u8] = b"Kaydara FBX Binary  \x00";
        if data.length() < Length::from(27u64) || !data.as_ref().starts_with(MAGIC) {
            let kind = if data.as_ref().starts_with(b"; FBX")
                || data.as_ref().starts_with(b"\xef\xbb\xbf; FBX")
            {
                FbxFormatViolation::Ascii
            } else {
                FbxFormatViolation::NotBinary
            };
            return Err(FbxDecodeError::Format(kind));
        }
        let version = FbxVersion::from(data.decode_u32(Offset::from(23u64)).map_err(
            |source: StorageBoundsError| -> FbxDecodeError {
                FbxDecodeError::Bounds {
                    section: FbxSection::Header,
                    source,
                }
            },
        )?);
        Ok(Self {
            data,
            position: Offset::from(27u64),
            limit: data.length().end_offset(),
            format: version.format(),
        })
    }
}
impl<'a> Reader<'a> {
    fn take(
        &mut self,
        section: FbxSection,
        length: Length,
    ) -> Result<StorageView<'a>, FbxDecodeError> {
        let requested = Span {
            offset: self.position,
            length,
        };
        let end = self.position.advance(length).map_err(
            |source: StorageBoundsError| -> FbxDecodeError {
                FbxDecodeError::Bounds { section, source }
            },
        )?;
        if end > self.limit {
            return Err(FbxDecodeError::Bounds {
                section,
                source: StorageBoundsError::Outside {
                    requested,
                    available: self
                        .limit
                        .distance_from(Offset::default())
                        .expect("nonnegative file address"),
                },
            });
        }
        let view = self.data.portion(requested).map_err(
            |source: StorageBoundsError| -> FbxDecodeError {
                FbxDecodeError::Bounds { section, source }
            },
        )?;
        self.position = end;
        Ok(view)
    }
    fn node(&mut self) -> Result<Option<Node>, FbxDecodeError> {
        let header = NodeHeader::from_reader(self)?;
        let name = FbxRecordName::from(self.take(FbxSection::NodeName, header.name)?);
        if header.end == Offset::default() {
            return Ok(None);
        }
        if header.end < self.position || header.end > self.limit {
            return Err(FbxDecodeError::NodeExtent {
                end: header.end,
                minimum: self.position,
                limit: self.limit,
            });
        }
        let outer_limit = self.limit;
        self.limit = header.end;
        let props = self.properties(&header)?;
        let mut children = Vec::new();
        while let Ok(end) = self.position.advance(self.format.record_width()) {
            if end > self.limit {
                break;
            }
            match self.node()? {
                Some(child) => children.push(child),
                None => break,
            }
        }
        self.position = header.end;
        self.limit = outer_limit;
        Ok(Some(Node {
            name,
            props,
            children,
        }))
    }
}
impl NodeHeader {
    fn from_reader(reader: &mut Reader<'_>) -> Result<Self, FbxDecodeError> {
        let mut words = [0u64; 3];
        for word in &mut words {
            let raw = reader.take(FbxSection::NodeMetadata, reader.format.word_width())?;
            *word = match reader.format {
                HeaderFormat::Narrow => u64::from(
                    raw.decode_u32(Offset::default())
                        .expect("admitted metadata word"),
                ),
                HeaderFormat::Wide => raw
                    .decode_u64(Offset::default())
                    .expect("admitted metadata word"),
            };
        }
        let name = reader
            .take(FbxSection::NodeMetadata, Length::from(1u64))?
            .as_ref()[0];
        Ok(Self {
            end: Offset::from(words[0]),
            count: FbxPropertyCount::from(words[1]),
            properties: Length::from(words[2]),
            name: Length::from(u64::from(name)),
        })
    }
}
/// Parses the top-level node list without interpreting object or animation roles.
pub fn parse(data: StorageView<'_>) -> Result<Vec<Node>, FbxDecodeError> {
    let mut reader = Reader::try_from(data)?;
    let mut roots = Vec::new();
    while let Ok(end) = reader.position.advance(reader.format.record_width()) {
        if end > reader.limit {
            break;
        }
        match reader.node()? {
            Some(node) => roots.push(node),
            None => break,
        }
    }
    Ok(roots)
}
