//! Whole-directory admission, including checked ZIP64 field substitution.

use super::{
    ArchiveMemberCount, ArchiveMemberName, Entry, Zip64Field, ZipChecksum, ZipCompression,
    ZipDirectoryError, ZipEndError, ZipProtection, ZipReadError, ZipRecordKind, ZipSignature,
};
use fabelgeist_storage::{
    StorageByteLength as Length, StorageByteOffset as Offset, StorageByteSpan as Span, StorageView,
};

pub(super) struct DirectoryLocation {
    count: ArchiveMemberCount,
    span: Span,
}
impl DirectoryLocation {
    pub(super) fn from_view(data: StorageView<'_>) -> Result<Self, ZipEndError> {
        let end = Self::end_position(data)?;
        let view = data.tail_at(end).map_err(ZipEndError::Bounds)?;
        ZipRecordKind::EndDirectory
            .admit(view)
            .map_err(ZipEndError::Record)?;
        let disk = view
            .decode_u16(Offset::from(4u64))
            .map_err(ZipEndError::Bounds)?;
        let directory_disk = view
            .decode_u16(Offset::from(6u64))
            .map_err(ZipEndError::Bounds)?;
        let disk_count = view
            .decode_u16(Offset::from(8u64))
            .map_err(ZipEndError::Bounds)?;
        let count = view
            .decode_u16(Offset::from(10u64))
            .map_err(ZipEndError::Bounds)?;
        if disk != 0 || directory_disk != 0 || disk_count != count {
            return Err(ZipEndError::MultipleDisks);
        }
        let length = view
            .decode_u32(Offset::from(12u64))
            .map_err(ZipEndError::Bounds)?;
        let offset = view
            .decode_u32(Offset::from(16u64))
            .map_err(ZipEndError::Bounds)?;
        let (location, stop) = if count == u16::MAX || length == u32::MAX || offset == u32::MAX {
            Self::zip64(data, end)?
        } else {
            (
                Self {
                    count: ArchiveMemberCount::from(u64::from(count)),
                    span: Span {
                        offset: Offset::from(u64::from(offset)),
                        length: Length::from(u64::from(length)),
                    },
                },
                end,
            )
        };
        data.portion(location.span).map_err(ZipEndError::Bounds)?;
        if location
            .span
            .offset
            .advance(location.span.length)
            .map_err(ZipEndError::Bounds)?
            > stop
        {
            return Err(ZipEndError::DirectoryAfterEnd);
        }
        Ok(location)
    }
    fn end_position(data: StorageView<'_>) -> Result<Offset, ZipEndError> {
        let minimum = ZipRecordKind::EndDirectory.width();
        let mut candidate = data
            .length()
            .end_offset()
            .preceding(minimum)
            .ok_or(ZipEndError::MissingEndRecord)?;
        let earliest = data
            .length()
            .end_offset()
            .preceding(Length::from(65_557u64))
            .unwrap_or_default();
        loop {
            let view = data.tail_at(candidate).map_err(ZipEndError::Bounds)?;
            let signature = ZipSignature::from(
                view.decode_u32(Offset::default())
                    .map_err(ZipEndError::Bounds)?,
            );
            if signature == ZipRecordKind::EndDirectory.signature() {
                let comment = Length::from(u64::from(
                    view.decode_u16(Offset::from(20u64))
                        .map_err(ZipEndError::Bounds)?,
                ));
                if candidate
                    .advance(minimum.checked_add(comment).map_err(ZipEndError::Bounds)?)
                    .map_err(ZipEndError::Bounds)?
                    == data.length().end_offset()
                {
                    return Ok(candidate);
                }
            }
            if candidate == earliest {
                break;
            }
            candidate = candidate
                .preceding(Length::from(1u64))
                .expect("position is above earliest");
        }
        Err(ZipEndError::MissingEndRecord)
    }
    fn zip64(data: StorageView<'_>, end: Offset) -> Result<(Self, Offset), ZipEndError> {
        let locator = end
            .preceding(ZipRecordKind::Zip64Locator.width())
            .ok_or(ZipEndError::MissingZip64Locator)?;
        let view = data.tail_at(locator).map_err(ZipEndError::Bounds)?;
        ZipRecordKind::Zip64Locator
            .admit(view)
            .map_err(ZipEndError::Record)?;
        if view
            .decode_u32(Offset::from(4u64))
            .map_err(ZipEndError::Bounds)?
            != 0
            || view
                .decode_u32(Offset::from(16u64))
                .map_err(ZipEndError::Bounds)?
                != 1
        {
            return Err(ZipEndError::MultipleDisks);
        }
        let offset = Offset::from(
            view.decode_u64(Offset::from(8u64))
                .map_err(ZipEndError::Bounds)?,
        );
        let record = data.tail_at(offset).map_err(ZipEndError::Bounds)?;
        ZipRecordKind::Zip64End
            .admit(record)
            .map_err(ZipEndError::Record)?;
        let length = Length::from(
            record
                .decode_u64(Offset::from(4u64))
                .map_err(ZipEndError::Bounds)?,
        );
        if length < Length::from(44u64) {
            return Err(ZipEndError::InvalidZip64RecordLength);
        }
        let complete_length = length
            .checked_add(Length::from(12u64))
            .map_err(ZipEndError::Bounds)?;
        data.portion(Span {
            offset,
            length: complete_length,
        })
        .map_err(ZipEndError::Bounds)?;
        if offset
            .advance(complete_length)
            .map_err(ZipEndError::Bounds)?
            > locator
        {
            return Err(ZipEndError::InvalidZip64RecordLength);
        }
        if record
            .decode_u32(Offset::from(16u64))
            .map_err(ZipEndError::Bounds)?
            != 0
            || record
                .decode_u32(Offset::from(20u64))
                .map_err(ZipEndError::Bounds)?
                != 0
            || record
                .decode_u64(Offset::from(24u64))
                .map_err(ZipEndError::Bounds)?
                != record
                    .decode_u64(Offset::from(32u64))
                    .map_err(ZipEndError::Bounds)?
        {
            return Err(ZipEndError::MultipleDisks);
        }
        Ok((
            Self {
                count: ArchiveMemberCount::from(
                    record
                        .decode_u64(Offset::from(32u64))
                        .map_err(ZipEndError::Bounds)?,
                ),
                span: Span {
                    offset: Offset::from(
                        record
                            .decode_u64(Offset::from(48u64))
                            .map_err(ZipEndError::Bounds)?,
                    ),
                    length: Length::from(
                        record
                            .decode_u64(Offset::from(40u64))
                            .map_err(ZipEndError::Bounds)?,
                    ),
                },
            },
            offset,
        ))
    }
    pub(super) fn entries(self, data: StorageView<'_>) -> Result<Vec<Entry>, ZipReadError> {
        let directory = data.portion(self.span).map_err(
            |source: fabelgeist_storage::StorageBoundsError| -> ZipReadError {
                ZipReadError::End(ZipEndError::Bounds(source))
            },
        )?;
        let mut position = Offset::default();
        let mut entries = Vec::new();
        for ordinal in self.count.ordinals() {
            let result = directory
                .tail_at(position)
                .map_err(ZipDirectoryError::Bounds)
                .and_then(Entry::from_central_record);
            let (entry, width) = result.map_err(|source: ZipDirectoryError| -> ZipReadError {
                ZipReadError::Directory { ordinal, source }
            })?;
            position = position.advance(width).map_err(
                |source: fabelgeist_storage::StorageBoundsError| -> ZipReadError {
                    ZipReadError::Directory {
                        ordinal,
                        source: ZipDirectoryError::Bounds(source),
                    }
                },
            )?;
            entries.push(entry);
        }
        Ok(entries)
    }
}
impl Entry {
    fn from_central_record(view: StorageView<'_>) -> Result<(Self, Length), ZipDirectoryError> {
        ZipRecordKind::CentralMember
            .admit(view)
            .map_err(ZipDirectoryError::Record)?;
        let name_length = Length::from(u64::from(
            view.decode_u16(Offset::from(28u64))
                .map_err(ZipDirectoryError::Bounds)?,
        ));
        let extra_length = Length::from(u64::from(
            view.decode_u16(Offset::from(30u64))
                .map_err(ZipDirectoryError::Bounds)?,
        ));
        let comment_length = Length::from(u64::from(
            view.decode_u16(Offset::from(32u64))
                .map_err(ZipDirectoryError::Bounds)?,
        ));
        let fixed = ZipRecordKind::CentralMember.width();
        let extra_start = fixed
            .checked_add(name_length)
            .map_err(ZipDirectoryError::Bounds)?;
        let width = extra_start
            .checked_add(extra_length)
            .and_then(
                |length: Length| -> Result<Length, fabelgeist_storage::StorageBoundsError> {
                    length.checked_add(comment_length)
                },
            )
            .map_err(ZipDirectoryError::Bounds)?;
        view.portion(Span {
            offset: Offset::default(),
            length: width,
        })
        .map_err(ZipDirectoryError::Bounds)?;
        let name = ArchiveMemberName::from(
            view.portion(Span {
                offset: fixed.end_offset(),
                length: name_length,
            })
            .map_err(ZipDirectoryError::Bounds)?
            .as_ref()
            .to_vec(),
        );
        let extra = view
            .portion(Span {
                offset: extra_start.end_offset(),
                length: extra_length,
            })
            .map_err(ZipDirectoryError::Bounds)?;
        let compressed = view
            .decode_u32(Offset::from(20u64))
            .map_err(ZipDirectoryError::Bounds)?;
        let uncompressed = view
            .decode_u32(Offset::from(24u64))
            .map_err(ZipDirectoryError::Bounds)?;
        let local = view
            .decode_u32(Offset::from(42u64))
            .map_err(ZipDirectoryError::Bounds)?;
        let mut extended = Zip64Extra::from_view(extra)?;
        let uncompressed_size = if uncompressed == u32::MAX {
            extended.length(Zip64Field::UncompressedLength)?
        } else {
            Length::from(u64::from(uncompressed))
        };
        let compressed_size = if compressed == u32::MAX {
            extended.length(Zip64Field::CompressedLength)?
        } else {
            Length::from(u64::from(compressed))
        };
        let local_header_offset = if local == u32::MAX {
            extended.offset()?
        } else {
            Offset::from(u64::from(local))
        };
        Ok((
            Self {
                name,
                compression: ZipCompression::from(
                    view.decode_u16(Offset::from(10u64))
                        .map_err(ZipDirectoryError::Bounds)?,
                ),
                protection: ZipProtection::from(
                    view.decode_u16(Offset::from(8u64))
                        .map_err(ZipDirectoryError::Bounds)?,
                ),
                checksum: ZipChecksum::from(
                    view.decode_u32(Offset::from(16u64))
                        .map_err(ZipDirectoryError::Bounds)?,
                ),
                compressed_size,
                uncompressed_size,
                local_header_offset,
            },
            width,
        ))
    }
}
struct Zip64Extra<'a> {
    view: Option<StorageView<'a>>,
    position: Offset,
}
impl<'a> Zip64Extra<'a> {
    fn from_view(extra: StorageView<'a>) -> Result<Self, ZipDirectoryError> {
        let mut position = Offset::default();
        while position < extra.length().end_offset() {
            let header = extra.tail_at(position).map_err(ZipDirectoryError::Bounds)?;
            let tag = header
                .decode_u16(Offset::default())
                .map_err(ZipDirectoryError::Bounds)?;
            let length = Length::from(u64::from(
                header
                    .decode_u16(Offset::from(2u64))
                    .map_err(ZipDirectoryError::Bounds)?,
            ));
            let body = position
                .advance(Length::from(4u64))
                .map_err(ZipDirectoryError::Bounds)?;
            let view = extra
                .portion(Span {
                    offset: body,
                    length,
                })
                .map_err(ZipDirectoryError::Bounds)?;
            if tag == 0x0001 {
                return Ok(Self {
                    view: Some(view),
                    position: Offset::default(),
                });
            }
            position = body.advance(length).map_err(ZipDirectoryError::Bounds)?;
        }
        Ok(Self {
            view: None,
            position: Offset::default(),
        })
    }
    fn length(&mut self, field: Zip64Field) -> Result<Length, ZipDirectoryError> {
        let view = self.view.ok_or(ZipDirectoryError::MissingZip64(field))?;
        let length = Length::from(
            view.decode_u64(self.position)
                .map_err(ZipDirectoryError::Bounds)?,
        );
        self.position = self
            .position
            .advance(Length::from(8u64))
            .map_err(ZipDirectoryError::Bounds)?;
        Ok(length)
    }
    fn offset(&mut self) -> Result<Offset, ZipDirectoryError> {
        let view = self.view.ok_or(ZipDirectoryError::MissingZip64(
            Zip64Field::LocalHeaderPosition,
        ))?;
        let offset = Offset::from(
            view.decode_u64(self.position)
                .map_err(ZipDirectoryError::Bounds)?,
        );
        self.position = self
            .position
            .advance(Length::from(8u64))
            .map_err(ZipDirectoryError::Bounds)?;
        Ok(offset)
    }
}
