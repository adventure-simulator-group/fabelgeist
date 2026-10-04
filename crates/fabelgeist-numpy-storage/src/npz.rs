//! Array lookup retains exact NPZ keys and concrete archive/array causes.

use crate::npy::{NpyArray, NpyDecodeError};
use crate::zip::{
    ArchiveMemberName, ArchiveMemberPresence, Entry, NpzArrayName, ZipArchive, ZipReadError,
};
use fabelgeist_fs::{FileContents, NativeFile};
use fabelgeist_storage::StorageByteLength;

#[derive(Debug)]
pub enum NpzArrayError {
    Missing(NpzArrayName),
    Read {
        name: NpzArrayName,
        source: Box<ZipReadError>,
    },
    Decode {
        name: NpzArrayName,
        source: NpyDecodeError,
    },
}
impl std::fmt::Display for NpzArrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(name) => write!(f, "archive has no array {name}"),
            Self::Read { name, source } => write!(f, "reading array {name}: {source}"),
            Self::Decode { name, source } => write!(f, "decoding array {name}: {source}"),
        }
    }
}
impl std::error::Error for NpzArrayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Missing(_) => None,
            Self::Read { source, .. } => Some(source.as_ref()),
            Self::Decode { source, .. } => Some(source),
        }
    }
}

pub struct Npz {
    archive: ZipArchive,
}
impl Npz {
    pub fn open(file: &NativeFile) -> Result<Self, ZipReadError> {
        Ok(Self {
            archive: ZipArchive::open(file)?,
        })
    }
    pub fn from_bytes(bytes: FileContents) -> Result<Self, ZipReadError> {
        Ok(Self {
            archive: ZipArchive::from_bytes(bytes)?,
        })
    }
    pub fn names(&self) -> impl Iterator<Item = &ArchiveMemberName> {
        self.archive.names()
    }
    pub fn keys(&self) -> impl Iterator<Item = NpzArrayName> {
        self.names().map(NpzArrayName::from)
    }
    fn entry(&self, name: &NpzArrayName) -> Option<&Entry> {
        self.archive.entries().find(|entry: &&Entry| -> bool {
            name.matches(&entry.name) == ArchiveMemberPresence::Present
        })
    }
    pub fn contains(&self, name: &NpzArrayName) -> ArchiveMemberPresence {
        match self.entry(name) {
            Some(_) => ArchiveMemberPresence::Present,
            None => ArchiveMemberPresence::Absent,
        }
    }
    pub fn uncompressed_size(&self, name: &NpzArrayName) -> Option<StorageByteLength> {
        self.entry(name)
            .and_then(|entry: &Entry| -> Option<StorageByteLength> {
                self.archive.uncompressed_size(&entry.name)
            })
    }
    pub fn array(&self, name: &NpzArrayName) -> Result<NpyArray, NpzArrayError> {
        let entry = self
            .entry(name)
            .ok_or_else(|| -> NpzArrayError { NpzArrayError::Missing(name.clone()) })?;
        let bytes =
            self.archive
                .entry_bytes(entry)
                .map_err(|source: ZipReadError| -> NpzArrayError {
                    NpzArrayError::Read {
                        name: name.clone(),
                        source: Box::new(source),
                    }
                })?;
        NpyArray::from_view(bytes.view()).map_err(|source: NpyDecodeError| -> NpzArrayError {
            NpzArrayError::Decode {
                name: name.clone(),
                source,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npy::fixture::{FixtureShape, NpyFixture};
    use crate::npy::{Dtype, NpyDimension};
    use crate::zip::fixture::{
        ArchiveFixture, FixtureCompression, FixtureDirectory, FixtureMember,
    };
    #[test]
    fn reads_stored_members() {
        let mut payload = Vec::new();
        for value in [1.0f32, 2.0, 3.0, 4.0] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        let mut indices = Vec::new();
        for value in [7i64, 9] {
            indices.extend_from_slice(&value.to_le_bytes());
        }
        let archive = ArchiveFixture::from_members(
            &[
                FixtureMember {
                    name: ArchiveMemberName::from("weights.npy"),
                    payload: FileContents::from(
                        NpyFixture::from_parts(
                            Dtype::F32.into(),
                            FixtureShape::from(vec![NpyDimension::from(2), NpyDimension::from(2)]),
                            FileContents::from(payload),
                        )
                        .unwrap(),
                    ),
                    compression: FixtureCompression::Stored,
                },
                FixtureMember {
                    name: ArchiveMemberName::from("indices.npy"),
                    payload: FileContents::from(
                        NpyFixture::from_parts(
                            Dtype::I64.into(),
                            FixtureShape::from(vec![NpyDimension::from(2)]),
                            FileContents::from(indices),
                        )
                        .unwrap(),
                    ),
                    compression: FixtureCompression::Stored,
                },
            ],
            FixtureDirectory::Ordinary,
        );

        let mapped = archive.mapped();
        let npz = Npz::open(&mapped.file).unwrap();
        assert_eq!(
            npz.keys().collect::<Vec<_>>(),
            [NpzArrayName::from("weights"), NpzArrayName::from("indices")]
        );
        assert_eq!(
            npz.contains(&NpzArrayName::from("weights")),
            ArchiveMemberPresence::Present
        );
        assert_eq!(
            npz.contains(&NpzArrayName::from("weights.npy")),
            ArchiveMemberPresence::Present
        );
        assert_eq!(
            npz.contains(&NpzArrayName::from("missing")),
            ArchiveMemberPresence::Absent
        );

        let weights = npz.array(&NpzArrayName::from("weights")).unwrap();
        assert_eq!(
            weights.shape().dimensions(),
            [
                crate::npy::NpyDimension::from(2),
                crate::npy::NpyDimension::from(2)
            ]
        );
        assert_eq!(
            Vec::<f32>::from(weights.floating_values()),
            [1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(
            Vec::<i64>::from(
                npz.array(&NpzArrayName::from("indices"))
                    .unwrap()
                    .integer_values()
            ),
            [7, 9]
        );

        let memory_npz = Npz::from_bytes(archive.contents).unwrap();
        assert_eq!(
            Vec::<f32>::from(
                memory_npz
                    .array(&NpzArrayName::from("weights"))
                    .unwrap()
                    .floating_values()
            ),
            [1.0, 2.0, 3.0, 4.0]
        );
    }

    #[test]
    fn reports_a_missing_member() {
        let archive = ArchiveFixture::from_members(
            &[FixtureMember {
                name: ArchiveMemberName::from("a.npy"),
                payload: FileContents::from(
                    NpyFixture::from_parts(
                        Dtype::F32.into(),
                        FixtureShape::from(vec![NpyDimension::from(1)]),
                        FileContents::from(vec![0; 4]),
                    )
                    .unwrap(),
                ),
                compression: FixtureCompression::Stored,
            }],
            FixtureDirectory::Ordinary,
        );
        let mapped = archive.mapped();
        let npz = Npz::open(&mapped.file).unwrap();
        assert!(npz.array(&NpzArrayName::from("b")).is_err());
    }
    #[test]
    fn array_lookup_keeps_nested_member_and_decode_errors() {
        let name = NpzArrayName::from("weights");
        let fixture = ArchiveFixture::from_members(
            &[FixtureMember {
                name: ArchiveMemberName::from("weights.npy"),
                payload: FileContents::from(b"not-numpy".to_vec()),
                compression: FixtureCompression::Stored,
            }],
            FixtureDirectory::Ordinary,
        );
        let archive = Npz::from_bytes(fixture.contents).unwrap();
        match archive.array(&name) {
            Err(NpzArrayError::Decode {
                name: original,
                source: NpyDecodeError::Magic,
            }) => assert_eq!(original, name),
            _ => panic!("expected named NumPy decode failure"),
        }
        let fixture = ArchiveFixture::from_members(
            &[FixtureMember {
                name: ArchiveMemberName::from("weights.npy"),
                payload: FileContents::from(vec![1, 2, 3]),
                compression: FixtureCompression::Stored,
            }],
            FixtureDirectory::Ordinary,
        );
        let archive =
            Npz::from_bytes(fixture.corrupted(crate::zip::fixture::FixtureFault::Checksum))
                .unwrap();
        match archive.array(&name) {
            Err(NpzArrayError::Read {
                name: original,
                source,
            }) => {
                assert_eq!(original, name);
                assert!(matches!(
                    *source,
                    ZipReadError::Member {
                        source: crate::zip::ZipMemberError::Checksum { .. },
                        ..
                    }
                ));
            }
            _ => panic!("expected named archive read failure"),
        }
    }
}
