//! Authored protocol fixtures, including stored, deflated, and ZIP64 records.
use super::ArchiveMemberName;
use fabelgeist_fs::{FileContents, NativeFile};
use std::io::Write;

#[derive(Clone, Copy)]
pub(crate) enum FixtureCompression {
    Stored,
    Deflated,
}
#[derive(Clone, Copy)]
pub(crate) enum FixtureDirectory {
    Ordinary,
    Zip64,
}
pub(crate) struct FixtureMember {
    pub name: ArchiveMemberName,
    pub payload: FileContents,
    pub compression: FixtureCompression,
}
#[derive(Clone, Copy)]
pub(crate) enum FixtureFault {
    DirectoryCount,
    NameLength,
    LocalOffset,
    Zip64Extra,
    CompressedLength,
    DecodedLength,
    Checksum,
    Encryption,
    Compression,
    Deflate,
}
pub(crate) struct ArchiveFixture {
    pub contents: FileContents,
    central: usize,
    end: usize,
}
impl ArchiveFixture {
    pub(crate) fn from_members(
        members: &[FixtureMember],
        directory_kind: FixtureDirectory,
    ) -> Self {
        let mut records = FixtureRecords {
            out: Vec::new(),
            directory: Vec::new(),
            directory_kind,
        };
        for member in members {
            records.append_member(member);
        }
        records.finish(members)
    }
    pub(crate) fn corrupted(self, fault: FixtureFault) -> FileContents {
        let mut bytes: Vec<u8> = self.contents.into();
        match fault {
            FixtureFault::DirectoryCount => {
                bytes[self.end + 8..self.end + 10].copy_from_slice(&2u16.to_le_bytes());
                bytes[self.end + 10..self.end + 12].copy_from_slice(&2u16.to_le_bytes());
            }
            FixtureFault::NameLength => {
                bytes[self.central + 28..self.central + 30].copy_from_slice(&u16::MAX.to_le_bytes())
            }
            FixtureFault::LocalOffset => bytes[self.central + 42..self.central + 46]
                .copy_from_slice(&(u32::MAX - 1).to_le_bytes()),
            FixtureFault::Zip64Extra => {
                let name_len = u16::from_le_bytes(
                    bytes[self.central + 28..self.central + 30]
                        .try_into()
                        .unwrap(),
                ) as usize;
                bytes[self.central + 46 + name_len..self.central + 48 + name_len]
                    .copy_from_slice(&2u16.to_le_bytes());
            }
            FixtureFault::CompressedLength => bytes[self.central + 20..self.central + 24]
                .copy_from_slice(&(u32::MAX - 1).to_le_bytes()),
            FixtureFault::DecodedLength => {
                bytes[self.central + 24..self.central + 28].copy_from_slice(&1u32.to_le_bytes())
            }
            FixtureFault::Checksum => bytes[self.central + 16] ^= 1,
            FixtureFault::Encryption => bytes[self.central + 8] = 1,
            FixtureFault::Compression => {
                bytes[self.central + 10..self.central + 12].copy_from_slice(&99u16.to_le_bytes())
            }
            FixtureFault::Deflate => {
                let name_len = u16::from_le_bytes(bytes[26..28].try_into().unwrap()) as usize;
                bytes[30 + name_len] = 0xff;
            }
        }
        FileContents::from(bytes)
    }
    pub(crate) fn mapped(&self) -> MappedFixture {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("fixture.npz");
        std::fs::write(&path, &self.contents).unwrap();
        MappedFixture {
            _directory: directory,
            file: NativeFile::from(path),
        }
    }
}
pub(crate) struct MappedFixture {
    _directory: tempfile::TempDir,
    pub file: NativeFile,
}

/// Serializes the member records and then the complete central directory.
struct FixtureRecords {
    out: Vec<u8>,
    directory: Vec<u8>,
    directory_kind: FixtureDirectory,
}
impl FixtureRecords {
    fn append_member(&mut self, member: &FixtureMember) {
        let Self {
            out,
            directory,
            directory_kind,
        } = self;
        let name = member.name.as_ref();
        let payload = member.payload.as_ref();
        let compressed = match member.compression {
            FixtureCompression::Stored => payload.to_vec(),
            FixtureCompression::Deflated => {
                let mut encoder =
                    flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(payload).unwrap();
                encoder.finish().unwrap()
            }
        };
        let method = match member.compression {
            FixtureCompression::Stored => 0u16,
            FixtureCompression::Deflated => 8,
        };
        let offset = u64::try_from(out.len()).unwrap();
        let crc = crc32fast::hash(payload);
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&method.to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&u32::try_from(compressed.len()).unwrap().to_le_bytes());
        out.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
        out.extend_from_slice(&u16::try_from(name.len()).unwrap().to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&compressed);
        let mut extra = Vec::new();
        let (compressed_field, uncompressed_field, offset_field) = match directory_kind {
            FixtureDirectory::Ordinary => (
                u32::try_from(compressed.len()).unwrap(),
                u32::try_from(payload.len()).unwrap(),
                u32::try_from(offset).unwrap(),
            ),
            FixtureDirectory::Zip64 => {
                extra.extend_from_slice(&1u16.to_le_bytes());
                extra.extend_from_slice(&24u16.to_le_bytes());
                extra.extend_from_slice(&u64::try_from(payload.len()).unwrap().to_le_bytes());
                extra.extend_from_slice(&u64::try_from(compressed.len()).unwrap().to_le_bytes());
                extra.extend_from_slice(&offset.to_le_bytes());
                (u32::MAX, u32::MAX, u32::MAX)
            }
        };
        directory.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        directory.extend_from_slice(&[20, 0, 20, 0]);
        directory.extend_from_slice(&0u16.to_le_bytes());
        directory.extend_from_slice(&method.to_le_bytes());
        directory.extend_from_slice(&[0; 4]);
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&compressed_field.to_le_bytes());
        directory.extend_from_slice(&uncompressed_field.to_le_bytes());
        directory.extend_from_slice(&u16::try_from(name.len()).unwrap().to_le_bytes());
        directory.extend_from_slice(&u16::try_from(extra.len()).unwrap().to_le_bytes());
        directory.extend_from_slice(&[0; 10]);
        directory.extend_from_slice(&offset_field.to_le_bytes());
        directory.extend_from_slice(name);
        directory.extend_from_slice(&extra);
    }
    fn finish(self, members: &[FixtureMember]) -> ArchiveFixture {
        let Self {
            mut out,
            directory,
            directory_kind,
        } = self;
        let central = out.len();
        let size = directory.len();
        out.extend_from_slice(&directory);
        if let FixtureDirectory::Zip64 = directory_kind {
            let record = out.len();
            out.extend_from_slice(&0x0606_4b50u32.to_le_bytes());
            out.extend_from_slice(&44u64.to_le_bytes());
            out.extend_from_slice(&[45, 0, 45, 0]);
            out.extend_from_slice(&[0; 8]);
            for _ in 0..2 {
                out.extend_from_slice(&u64::try_from(members.len()).unwrap().to_le_bytes());
            }
            out.extend_from_slice(&u64::try_from(size).unwrap().to_le_bytes());
            out.extend_from_slice(&u64::try_from(central).unwrap().to_le_bytes());
            out.extend_from_slice(&0x0706_4b50u32.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(&u64::try_from(record).unwrap().to_le_bytes());
            out.extend_from_slice(&1u32.to_le_bytes());
        }
        let end = out.len();
        out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        out.extend_from_slice(&[0; 4]);
        match directory_kind {
            FixtureDirectory::Ordinary => {
                for _ in 0..2 {
                    out.extend_from_slice(&u16::try_from(members.len()).unwrap().to_le_bytes());
                }
                out.extend_from_slice(&u32::try_from(size).unwrap().to_le_bytes());
                out.extend_from_slice(&u32::try_from(central).unwrap().to_le_bytes());
            }
            FixtureDirectory::Zip64 => {
                out.extend_from_slice(&[0xff; 12]);
            }
        }
        out.extend_from_slice(&0u16.to_le_bytes());
        ArchiveFixture {
            contents: FileContents::from(out),
            central,
            end,
        }
    }
}
