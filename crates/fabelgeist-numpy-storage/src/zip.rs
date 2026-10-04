//! Read-only stored/deflated ZIP archives with checked ZIP64 metadata.
//! Stored members borrow the input mapping; member names retain exact bytes.

use fabelgeist_fs::{FileContents, NativeFile};
use fabelgeist_storage::{StorageByteLength, StorageByteOffset, StorageView};
use memmap2::Mmap;

mod directory;
mod error;
mod member;
mod names;
mod wire;
pub use error::{
    ArchiveFileOperation, Zip64Field, ZipDirectoryError, ZipEndError, ZipMemberError, ZipReadError,
};
pub use member::ZipMemberContents;
pub use names::{ArchiveMemberName, ArchiveMemberPresence, NpzArrayName};
pub use wire::{
    ArchiveMemberCount, ArchiveMemberOrdinal, ZipChecksum, ZipCompressionCode, ZipRecordError,
    ZipRecordKind, ZipSignature,
};
use wire::{ZipCompression, ZipProtection};

enum ArchiveData {
    Mapped(Mmap),
    Owned(FileContents),
}
impl ArchiveData {
    fn view(&self) -> StorageView<'_> {
        match self {
            Self::Mapped(data) => StorageView::from(data.as_ref()),
            Self::Owned(data) => StorageView::from(data.as_ref()),
        }
    }
}
pub(crate) struct Entry {
    pub name: ArchiveMemberName,
    compression: ZipCompression,
    protection: ZipProtection,
    checksum: ZipChecksum,
    compressed_size: StorageByteLength,
    uncompressed_size: StorageByteLength,
    local_header_offset: StorageByteOffset,
}
pub struct ZipArchive {
    data: ArchiveData,
    entries: Vec<Entry>,
}
impl ZipArchive {
    pub fn open(file: &NativeFile) -> Result<Self, ZipReadError> {
        let input = std::fs::File::open(file.as_ref()).map_err(
            |source: std::io::Error| -> ZipReadError {
                ZipReadError::Native {
                    file: file.clone(),
                    operation: ArchiveFileOperation::Open,
                    source,
                }
            },
        )?;
        // SAFETY: model assets are read-only inputs; concurrent truncation by
        // another process is outside the asset-loading contract.
        let data =
            unsafe { Mmap::map(&input) }.map_err(|source: std::io::Error| -> ZipReadError {
                ZipReadError::Native {
                    file: file.clone(),
                    operation: ArchiveFileOperation::Map,
                    source,
                }
            })?;
        Self::from_data(ArchiveData::Mapped(data))
    }
    pub fn from_bytes(data: FileContents) -> Result<Self, ZipReadError> {
        Self::from_data(ArchiveData::Owned(data))
    }
    fn from_data(data: ArchiveData) -> Result<Self, ZipReadError> {
        let location =
            directory::DirectoryLocation::from_view(data.view()).map_err(ZipReadError::End)?;
        let entries = location.entries(data.view())?;
        Ok(Self { data, entries })
    }
    pub fn names(&self) -> impl Iterator<Item = &ArchiveMemberName> {
        self.entries
            .iter()
            .map(|entry: &Entry| -> &ArchiveMemberName { &entry.name })
    }
    pub fn contains(&self, name: &ArchiveMemberName) -> ArchiveMemberPresence {
        match self.find(name) {
            Some(_) => ArchiveMemberPresence::Present,
            None => ArchiveMemberPresence::Absent,
        }
    }
    pub fn uncompressed_size(&self, name: &ArchiveMemberName) -> Option<StorageByteLength> {
        self.find(name)
            .map(|entry: &Entry| -> StorageByteLength { entry.uncompressed_size })
    }
    fn find(&self, name: &ArchiveMemberName) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|entry: &&Entry| -> bool { &entry.name == name })
    }
    pub(crate) fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }
    pub fn bytes(&self, name: &ArchiveMemberName) -> Result<ZipMemberContents<'_>, ZipReadError> {
        let entry = self
            .find(name)
            .ok_or_else(|| -> ZipReadError { ZipReadError::MissingMember(name.clone()) })?;
        self.entry_bytes(entry)
    }
    pub(crate) fn entry_bytes(&self, entry: &Entry) -> Result<ZipMemberContents<'_>, ZipReadError> {
        entry
            .decode(self.data.view())
            .map_err(|source: ZipMemberError| -> ZipReadError {
                ZipReadError::Member {
                    name: entry.name.clone(),
                    source,
                }
            })
    }
}

#[cfg(test)]
pub(crate) mod fixture;
#[cfg(test)]
mod tests;
