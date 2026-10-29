//! Read-only ZIP storage with application-owned member selection and decoding.
//!
//! `rawzip` owns directory, ZIP64 and member framing. Stored members borrow the
//! mapped or owned archive bytes; `flate2` decodes Deflate members into owned data.

use std::borrow::Cow;
use std::io::Read;
use std::path::Path;

use flate2::read::DeflateDecoder;
use memmap2::Mmap;
use rawzip::{ZipArchiveEntryWayfinder, ZipLocator, ZipSliceArchive};

pub use compression::ZipCompressionCode;
pub use error::{Result, ZipFileOperation, ZipReadError};
pub use extent::DeclaredUncompressedByteLength;

use compression::ZipCompression;
use member_count::ArchiveMemberCount;
pub use names::ArchiveMemberName;

mod compression;
mod error;
mod extent;
mod member_count;
mod names;

// Application selection policy, measured in archive bytes from the end.
const END_DIRECTORY_SEARCH_WINDOW_BYTES: u64 = 66_000;

struct Entry {
    name: ArchiveMemberName,
    compression: ZipCompression,
    uncompressed_size: DeclaredUncompressedByteLength,
    location: ZipArchiveEntryWayfinder,
}

/// A ZIP archive backed by a read-only mapping or an owned byte buffer.
pub struct ZipArchive {
    archive: ZipSliceArchive<ArchiveData>,
    entries: Vec<Entry>,
}

/// A member selected from, and inseparable from, its owning archive.
pub(crate) struct ZipMember<'archive> {
    archive: &'archive ZipArchive,
    entry: &'archive Entry,
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
        let archive = ZipLocator::new()
            .max_search_space(END_DIRECTORY_SEARCH_WINDOW_BYTES)
            .locate_in_slice(data)
            .map_err(|(_, source)| ZipReadError::from(source))?;
        let member_count = ArchiveMemberCount::from(archive.entries_hint());
        let mut entries = Vec::with_capacity(member_count.initial_capacity());
        let mut directory = archive.entries();
        // The declaration remains our upper bound; rawzip bounds each record
        // by the selected directory span. No partial archive is published on error.
        for _ in member_count.native_iteration_range() {
            let Some(header) = directory.next_entry().map_err(ZipReadError::from)? else {
                break;
            };
            entries.push(Entry {
                name: ArchiveMemberName::from(header.file_path().as_ref()),
                compression: ZipCompression::from(header.compression_method()),
                uncompressed_size: DeclaredUncompressedByteLength::from_bytes(
                    header.uncompressed_size_hint(),
                ),
                location: header.wayfinder(),
            });
        }
        Ok(Self { archive, entries })
    }

    /// Member names, in central-directory order.
    pub fn names(&self) -> impl Iterator<Item = &ArchiveMemberName> {
        self.entries.iter().map(|entry| &entry.name)
    }

    pub fn contains(&self, name: &ArchiveMemberName) -> bool {
        self.entries.iter().any(|entry| entry.name == *name)
    }

    /// Declared decoded length of the first exact match, without verification.
    pub fn uncompressed_size(
        &self,
        name: &ArchiveMemberName,
    ) -> Option<DeclaredUncompressedByteLength> {
        self.find(|member| member == name)
            .map(|member| member.uncompressed_size())
    }

    pub(crate) fn find(
        &self,
        predicate: impl Fn(&ArchiveMemberName) -> bool,
    ) -> Option<ZipMember<'_>> {
        self.entries
            .iter()
            .find(|entry| predicate(&entry.name))
            .map(|entry| ZipMember {
                archive: self,
                entry,
            })
    }

    /// Decodes one member. Stored members borrow from the archive buffer.
    pub fn bytes(&self, name: &ArchiveMemberName) -> Result<Cow<'_, [u8]>> {
        self.find(|member| member == name)
            .ok_or_else(|| ZipReadError::MissingMember { name: name.clone() })?
            .bytes()
    }
}

impl<'archive> ZipMember<'archive> {
    pub(crate) fn name(&self) -> &ArchiveMemberName {
        &self.entry.name
    }

    pub(crate) fn uncompressed_size(&self) -> DeclaredUncompressedByteLength {
        self.entry.uncompressed_size
    }

    pub(crate) fn bytes(&self) -> Result<Cow<'archive, [u8]>> {
        let entry = self.entry;
        let member = self
            .archive
            .archive
            .get_entry(entry.location)
            .map_err(|source| ZipReadError::member(entry.name.clone(), source))?;
        let raw = member.data();
        match entry.compression {
            ZipCompression::Stored => Ok(Cow::Borrowed(raw)),
            ZipCompression::Deflated => {
                let mut out =
                    Vec::with_capacity(entry.uncompressed_size.native_allocation_capacity());
                DeflateDecoder::new(raw)
                    .read_to_end(&mut out)
                    .map_err(|source| ZipReadError::Inflate {
                        name: entry.name.clone(),
                        source,
                    })?;
                Ok(Cow::Owned(out))
            }
            ZipCompression::Unsupported(code) => Err(ZipReadError::UnsupportedCompression { code }),
        }
    }
}
