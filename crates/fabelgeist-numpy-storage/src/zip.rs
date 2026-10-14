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

mod error;
pub use error::{Result, ZipFileOperation, ZipReadError};
use flate2::read::DeflateDecoder;
use memmap2::Mmap;

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

const END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const CENTRAL_FILE_HEADER: u32 = 0x0201_4b50;
const LOCAL_FILE_HEADER: u32 = 0x0403_4b50;
const ZIP64_END_OF_CENTRAL_DIRECTORY: u32 = 0x0606_4b50;
const ZIP64_LOCATOR: u32 = 0x0706_4b50;

pub(crate) struct Entry {
    pub name: String,
    compression: u16,
    compressed_size: u64,
    uncompressed_size: u64,
    local_header_offset: u64,
}

/// A memory-mapped zip archive.
pub struct ZipArchive {
    data: ArchiveData,
    entries: Vec<Entry>,
}

// Native little-endian scalar ports: each slice has its fixed array arity.
// Slice bounds and native offset arithmetic retain the reader's existing policy.
fn u16_at(data: &[u8], offset: usize) -> u16 {
    let mut scalar = [0; 2];
    scalar.copy_from_slice(&data[offset..offset + 2]);
    u16::from_le_bytes(scalar)
}

fn u32_at(data: &[u8], offset: usize) -> u32 {
    let mut scalar = [0; 4];
    scalar.copy_from_slice(&data[offset..offset + 4]);
    u32::from_le_bytes(scalar)
}

fn u64_at(data: &[u8], offset: usize) -> u64 {
    let mut scalar = [0; 8];
    scalar.copy_from_slice(&data[offset..offset + 8]);
    u64::from_le_bytes(scalar)
}

/// Replaces saturated 32-bit sizes/offsets with their zip64 values.
fn apply_zip64_extra(extra: &[u8], entry: &mut Entry) {
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
            for slot in [
                &mut entry.uncompressed_size,
                &mut entry.compressed_size,
                &mut entry.local_header_offset,
            ] {
                if *slot == u32::MAX as u64 && cursor + 8 <= body + size {
                    *slot = u64_at(extra, cursor);
                    cursor += 8;
                }
            }
            return;
        }
        offset = body + size;
    }
}

impl ZipArchive {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = std::fs::File::open(path).map_err(|source| ZipReadError::Native {
            operation: ZipFileOperation::Open,
            native_path: path.to_path_buf(),
            source,
        })?;
        // SAFETY: model assets are read-only inputs; concurrent truncation by
        // another process is out of scope, as elsewhere in this workspace.
        let data = unsafe { Mmap::map(&file) }.map_err(|source| ZipReadError::Native {
            operation: ZipFileOperation::Map,
            native_path: path.to_path_buf(),
            source,
        })?;
        Self::from_data(ArchiveData::Mapped(data))
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        Self::from_data(ArchiveData::Owned(data))
    }

    fn from_data(data: ArchiveData) -> Result<Self> {
        // Decode directory words directly at native allocation/range ports.
        // Count/offset ownership is separate; no primitive tuple crosses an API.
        // The end-of-central-directory record is within 64 KiB of the file end.
        let earliest = data.len().saturating_sub(66_000);
        let mut eocd = None;
        for offset in (earliest..data.len().saturating_sub(21)).rev() {
            if u32_at(data.as_ref(), offset) == END_OF_CENTRAL_DIRECTORY {
                eocd = Some(offset);
                break;
            }
        }
        let eocd = eocd.ok_or(ZipReadError::MissingEndRecord)?;
        let mut count = u16_at(data.as_ref(), eocd + 10) as u64;
        let mut offset = u32_at(data.as_ref(), eocd + 16) as u64;

        if (count == u16::MAX as u64 || offset == u32::MAX as u64) && eocd >= 20 {
            let locator = eocd - 20;
            if u32_at(data.as_ref(), locator) == ZIP64_LOCATOR {
                let record = u64_at(data.as_ref(), locator + 8) as usize;
                if record + 56 <= data.len()
                    && u32_at(data.as_ref(), record) == ZIP64_END_OF_CENTRAL_DIRECTORY
                {
                    count = u64_at(data.as_ref(), record + 32);
                    offset = u64_at(data.as_ref(), record + 48);
                }
            }
        }
        let mut entries = Vec::with_capacity(count.min(4096) as usize);
        for _ in 0..count {
            let base = offset as usize;
            if base + 46 > data.len() || u32_at(&data, base) != CENTRAL_FILE_HEADER {
                break;
            }
            let name_len = u16_at(&data, base + 28) as usize;
            let extra_len = u16_at(&data, base + 30) as usize;
            let comment_len = u16_at(&data, base + 32) as usize;
            let mut entry = Entry {
                name: String::from_utf8_lossy(&data[base + 46..base + 46 + name_len]).into_owned(),
                compression: u16_at(&data, base + 10),
                compressed_size: u32_at(&data, base + 20) as u64,
                uncompressed_size: u32_at(&data, base + 24) as u64,
                local_header_offset: u32_at(&data, base + 42) as u64,
            };
            apply_zip64_extra(
                &data[base + 46 + name_len..base + 46 + name_len + extra_len],
                &mut entry,
            );
            entries.push(entry);
            offset += (46 + name_len + extra_len + comment_len) as u64;
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

    /// Size of a member once decoded, without decoding it.
    pub fn uncompressed_size(&self, name: &str) -> Option<u64> {
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
            .ok_or_else(|| ZipReadError::MissingMember {
                native_query: name.to_owned(),
            })?;
        self.entry_bytes(entry)
    }

    pub(crate) fn entry_bytes(&self, entry: &Entry) -> Result<Cow<'_, [u8]>> {
        let base = entry.local_header_offset as usize;
        if base + 30 > self.data.len() || u32_at(&self.data, base) != LOCAL_FILE_HEADER {
            return Err(ZipReadError::CorruptMember {
                native_name: entry.name.clone(),
            });
        }
        // The local header repeats the name and may carry a different extra field.
        let name_len = u16_at(&self.data, base + 26) as usize;
        let extra_len = u16_at(&self.data, base + 28) as usize;
        let start = base + 30 + name_len + extra_len;
        let end = start + entry.compressed_size as usize;
        if end > self.data.len() {
            return Err(ZipReadError::MemberPastEnd {
                native_name: entry.name.clone(),
            });
        }
        let raw = &self.data[start..end];

        match entry.compression {
            0 => Ok(Cow::Borrowed(raw)),
            8 => {
                let mut out = Vec::with_capacity(entry.uncompressed_size as usize);
                DeflateDecoder::new(raw)
                    .read_to_end(&mut out)
                    .map_err(|source| ZipReadError::Inflate {
                        native_name: entry.name.clone(),
                        source,
                    })?;
                Ok(Cow::Owned(out))
            }
            other => Err(ZipReadError::UnsupportedCompression { native_code: other }),
        }
    }
}
