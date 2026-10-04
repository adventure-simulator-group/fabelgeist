//! Member payload decoding and integrity admission.

use super::{Entry, ZipChecksum, ZipCompression, ZipMemberError, ZipProtection, ZipRecordKind};
use fabelgeist_fs::FileContents;
use fabelgeist_storage::{
    StorageByteLength as Length, StorageByteOffset as Offset, StorageByteSpan as Span, StorageView,
};
use flate2::read::DeflateDecoder;
use std::io::Read;

#[derive(Debug)]
enum MemberPayload<'a> {
    Stored(StorageView<'a>),
    Inflated(FileContents),
}
#[derive(Debug)]
pub struct ZipMemberContents<'a> {
    data: MemberPayload<'a>,
}
impl ZipMemberContents<'_> {
    pub fn view(&self) -> StorageView<'_> {
        match &self.data {
            MemberPayload::Stored(view) => *view,
            MemberPayload::Inflated(contents) => StorageView::from(contents.as_ref()),
        }
    }
}
impl Entry {
    pub(super) fn decode<'a>(
        &self,
        data: StorageView<'a>,
    ) -> Result<ZipMemberContents<'a>, ZipMemberError> {
        let local = data
            .tail_at(self.local_header_offset)
            .map_err(ZipMemberError::Bounds)?;
        ZipRecordKind::LocalMember
            .admit(local)
            .map_err(ZipMemberError::Record)?;
        if self.protection == ZipProtection::Encrypted {
            return Err(ZipMemberError::Encrypted);
        }
        let name_length = Length::from(u64::from(
            local
                .decode_u16(Offset::from(26u64))
                .map_err(ZipMemberError::Bounds)?,
        ));
        let extra_length = Length::from(u64::from(
            local
                .decode_u16(Offset::from(28u64))
                .map_err(ZipMemberError::Bounds)?,
        ));
        let start = ZipRecordKind::LocalMember
            .width()
            .checked_add(name_length)
            .and_then(
                |length: Length| -> Result<Length, fabelgeist_storage::StorageBoundsError> {
                    length.checked_add(extra_length)
                },
            )
            .map_err(ZipMemberError::Bounds)?;
        let raw = local
            .portion(Span {
                offset: start.end_offset(),
                length: self.compressed_size,
            })
            .map_err(ZipMemberError::Bounds)?;
        let contents = match self.compression {
            ZipCompression::Stored => MemberPayload::Stored(raw),
            ZipCompression::Deflated => {
                let mut out = Vec::new();
                DeflateDecoder::new(raw.as_ref())
                    .take(u64::from(
                        self.uncompressed_size
                            .checked_add(Length::from(1u64))
                            .map_err(ZipMemberError::Bounds)?,
                    ))
                    .read_to_end(&mut out)
                    .map_err(ZipMemberError::Inflate)?;
                MemberPayload::Inflated(FileContents::from(out))
            }
            ZipCompression::Unsupported(code) => {
                return Err(ZipMemberError::UnsupportedCompression(code));
            }
        };
        let contents = ZipMemberContents { data: contents };
        let actual = contents.view().length();
        if actual != self.uncompressed_size {
            return Err(ZipMemberError::Size {
                expected: self.uncompressed_size,
                actual,
            });
        }
        let actual = ZipChecksum::from(contents.view());
        if actual != self.checksum {
            return Err(ZipMemberError::Checksum {
                expected: self.checksum,
                actual,
            });
        }
        Ok(contents)
    }
}
