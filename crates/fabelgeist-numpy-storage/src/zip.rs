//! A minimal read-only zip archive reader.
//!
//! Stored and deflated members, zip64 included. Lives here because `.npz` is a
//! zip of `.npy` members, and is public because PyTorch archives are zips too:
//! `burn-torch-storage` reads `.pt`/`.ckpt` through this same code rather than
//! carrying a second copy of the central-directory parser.
//!
//! Stored members are handed back as borrowed slices of the mapping, so reading
//! a multi-gigabyte tensor storage costs no copy.

use std::borrow::Cow;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};
use flate2::read::DeflateDecoder;
use memmap2::Mmap;

pub use extent::{ArchiveByteOffset, CompressedByteLength, DeclaredUncompressedByteLength};

use compression::ZipCompression;
use member_count::ArchiveMemberCount;
use record_kind::ZipRecordKind;

mod compression;
mod extent;
mod member_count;
mod record_kind;

/// Selected directory metadata with distinct count and archive-byte addressing.
struct CentralDirectory {
    member_count: ArchiveMemberCount,
    byte_offset: ArchiveByteOffset,
}

pub(crate) struct Entry {
    pub name: String,
    compression: ZipCompression,
    compressed_size: CompressedByteLength,
    uncompressed_size: DeclaredUncompressedByteLength,
    local_header_offset: ArchiveByteOffset,
}

/// A memory-mapped zip archive.
pub struct ZipArchive {
    data: ArchiveData,
    entries: Vec<Entry>,
}

enum ArchiveData {
    Mapped(Mmap),
    Owned(Vec<u8>),
}

impl AsRef<[u8]> for ArchiveData {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Mapped(data) => data,
            Self::Owned(data) => data,
        }
    }
}

impl std::ops::Deref for ArchiveData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl CentralDirectory {
    /// Locates the central directory, following the zip64 locator when present.
    fn locate(data: &[u8]) -> Result<Self> {
        // The end-of-central-directory record is within 64 KiB of the file end.
        let earliest = data.len().saturating_sub(66_000);
        let mut eocd = None;
        for offset in (earliest..data.len().saturating_sub(21)).rev() {
            if ZipRecordKind::read_at(data, offset) == ZipRecordKind::EndDirectory {
                eocd = Some(offset);
                break;
            }
        }
        let eocd = eocd.context("not a zip archive: no end-of-central-directory record")?;
        let mut count = ArchiveMemberCount::from(u16_at(data, eocd + 10));
        let mut offset = ArchiveByteOffset::from_archive_start(u64::from(u32_at(data, eocd + 16)));

        if (count.is_saturated_ordinary() || offset.is_saturated_ordinary()) && eocd >= 20 {
            let locator = eocd - 20;
            if ZipRecordKind::read_at(data, locator) == ZipRecordKind::Zip64Locator {
                let record = ArchiveByteOffset::from_archive_start(u64_at(data, locator + 8))
                    .native_byte_index();
                if record + 56 <= data.len()
                    && ZipRecordKind::read_at(data, record) == ZipRecordKind::Zip64End
                {
                    count = ArchiveMemberCount::from(u64_at(data, record + 32));
                    offset = ArchiveByteOffset::from_archive_start(u64_at(data, record + 48));
                }
            }
        }
        Ok(Self {
            member_count: count,
            byte_offset: offset,
        })
    }
}

impl Entry {
    /// Replaces saturated fields in uncompressed/compressed/local-offset order.
    fn apply_zip64_extra(&mut self, extra: &[u8]) {
        let mut offset = 0;
        while offset + 4 <= extra.len() {
            let id = u16_at(extra, offset);
            let size = u16_at(extra, offset + 2) as usize;
            let body = offset + 4;
            if body + size > extra.len() {
                return;
            }
            if id == 0x0001 {
                let mut cursor = body;
                if self.uncompressed_size.is_saturated_ordinary() && cursor + 8 <= body + size {
                    self.uncompressed_size =
                        DeclaredUncompressedByteLength::from_bytes(u64_at(extra, cursor));
                    cursor += 8;
                }
                if self.compressed_size.is_saturated_ordinary() && cursor + 8 <= body + size {
                    self.compressed_size = CompressedByteLength::from_bytes(u64_at(extra, cursor));
                    cursor += 8;
                }
                if self.local_header_offset.is_saturated_ordinary() && cursor + 8 <= body + size {
                    self.local_header_offset =
                        ArchiveByteOffset::from_archive_start(u64_at(extra, cursor));
                }
                return;
            }
            offset = body + size;
        }
    }
}

impl ZipArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file =
            std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
        // SAFETY: model assets are read-only inputs; concurrent truncation by
        // another process is out of scope, as elsewhere in this workspace.
        let data = unsafe { Mmap::map(&file) }
            .with_context(|| format!("memory-mapping {}", path.display()))?;
        Self::from_data(ArchiveData::Mapped(data))
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        Self::from_data(ArchiveData::Owned(data))
    }

    fn from_data(data: ArchiveData) -> Result<Self> {
        let CentralDirectory {
            member_count,
            byte_offset: mut offset,
        } = CentralDirectory::locate(data.as_ref())?;
        let mut entries = Vec::with_capacity(member_count.initial_capacity());
        for _ in member_count.native_iteration_range() {
            let base = offset.native_byte_index();
            if base + 46 > data.len()
                || ZipRecordKind::read_at(&data, base) != ZipRecordKind::CentralMember
            {
                break;
            }
            let name_len = u16_at(&data, base + 28) as usize;
            let extra_len = u16_at(&data, base + 30) as usize;
            let comment_len = u16_at(&data, base + 32) as usize;
            let mut entry = Entry {
                name: String::from_utf8_lossy(&data[base + 46..base + 46 + name_len]).into_owned(),
                compression: ZipCompression::from(u16_at(&data, base + 10)),
                compressed_size: CompressedByteLength::from_bytes(u64::from(u32_at(
                    &data,
                    base + 20,
                ))),
                uncompressed_size: DeclaredUncompressedByteLength::from_bytes(u64::from(u32_at(
                    &data,
                    base + 24,
                ))),
                local_header_offset: ArchiveByteOffset::from_archive_start(u64::from(u32_at(
                    &data,
                    base + 42,
                ))),
            };
            entry.apply_zip64_extra(&data[base + 46 + name_len..base + 46 + name_len + extra_len]);
            entries.push(entry);
            offset.advance_native_bytes(46 + name_len + extra_len + comment_len);
        }

        Ok(Self { data, entries })
    }

    /// Member names, in central-directory order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|entry| entry.name.as_str())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }

    /// Declared decoded byte length, without decoding or verifying the payload.
    pub fn uncompressed_size(&self, name: &str) -> Option<DeclaredUncompressedByteLength> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.uncompressed_size)
    }

    pub(crate) fn find(&self, predicate: impl Fn(&str) -> bool) -> Option<&Entry> {
        self.entries.iter().find(|entry| predicate(&entry.name))
    }

    /// Decodes one member. Stored members borrow from the mapping.
    pub fn bytes(&self, name: &str) -> Result<Cow<'_, [u8]>> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.name == name)
            .with_context(|| format!("archive has no member {name:?}"))?;
        self.entry_bytes(entry)
    }

    pub(crate) fn entry_bytes(&self, entry: &Entry) -> Result<Cow<'_, [u8]>> {
        let base = entry.local_header_offset.native_byte_index();
        if base + 30 > self.data.len()
            || ZipRecordKind::read_at(&self.data, base) != ZipRecordKind::LocalMember
        {
            bail!("corrupt zip entry {}", entry.name);
        }
        // The local header repeats the name and may carry a different extra field.
        let name_len = u16_at(&self.data, base + 26) as usize;
        let extra_len = u16_at(&self.data, base + 28) as usize;
        let start = base + 30 + name_len + extra_len;
        let end = start + entry.compressed_size.native_slice_length();
        if end > self.data.len() {
            bail!("zip entry {} runs past the end of the archive", entry.name);
        }
        let raw = &self.data[start..end];

        match entry.compression {
            ZipCompression::Stored => Ok(Cow::Borrowed(raw)),
            ZipCompression::Deflated => {
                let mut out =
                    Vec::with_capacity(entry.uncompressed_size.native_allocation_capacity());
                DeflateDecoder::new(raw)
                    .read_to_end(&mut out)
                    .with_context(|| format!("inflating zip entry {}", entry.name))?;
                Ok(Cow::Owned(out))
            }
            ZipCompression::Unsupported(code) => bail!("unsupported zip compression method {code}"),
        }
    }
}

fn u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(data[offset..offset + 2].try_into().unwrap())
}

fn u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn u64_at(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}
