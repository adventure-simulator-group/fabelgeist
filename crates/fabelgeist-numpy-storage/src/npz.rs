//! `.npz` archive reading (a zip of `.npy` members).
//!
//! `np.savez` writes stored members and `np.savez_compressed` writes deflated
//! ones; both are handled, as is zip64, which NumPy emits for large arrays. The
//! archive plumbing itself lives in [`crate::zip`].

use std::path::Path;

use anyhow::{Context, Result};

use crate::npy::NpyArray;
use crate::zip::{ArchiveMemberName, DeclaredUncompressedByteLength, ZipArchive, ZipMember};
pub use name::NpzArrayName;

mod name;

/// An `.npz` archive backed by a read-only mapping or owned bytes.
pub struct Npz {
    archive: ZipArchive,
}

/// A selected NPZ member tied to the archive that owns its bytes.
///
/// Metadata and decoding use this exact first match. The original query is
/// retained for parsing diagnostics; a present member may still fail to decode.
pub struct NpzMember<'a> {
    member: ZipMember<'a>,
    query: NpzArrayName,
}

impl Npz {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            archive: ZipArchive::open(path)?,
        })
    }

    /// Opens an archive already loaded in memory, for browser and streamed assets.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            archive: ZipArchive::from_bytes(bytes)?,
        })
    }

    /// Parsed member names in central-directory order, retaining their suffix.
    pub fn names(&self) -> impl Iterator<Item = &ArchiveMemberName> {
        self.archive.names()
    }

    /// Array keys in member order, each with one trailing `.npy` removed.
    pub fn keys(&self) -> impl Iterator<Item = NpzArrayName> {
        self.names().map(NpzArrayName::from)
    }

    /// Selects the first full-name or single-suffix match in directory order.
    pub fn get(&self, name: &NpzArrayName) -> Option<NpzMember<'_>> {
        self.archive
            .find(|member| name.matches(member))
            .map(|member| NpzMember {
                member,
                query: name.clone(),
            })
    }

    /// Requires a member and decodes it, reporting absence as a lookup error.
    pub fn array(&self, name: &NpzArrayName) -> Result<NpyArray> {
        self.get(name)
            .with_context(|| format!("archive has no member {name:?}"))?
            .array()
    }
}

impl NpzMember<'_> {
    pub fn name(&self) -> &ArchiveMemberName {
        self.member.name()
    }

    /// The directory's declared size, without payload verification.
    pub fn uncompressed_size(&self) -> DeclaredUncompressedByteLength {
        self.member.uncompressed_size()
    }

    pub fn array(&self) -> Result<NpyArray> {
        let bytes = self.member.bytes()?;
        NpyArray::from_bytes(&bytes).with_context(|| format!("reading array {:?}", self.query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
    const CENTRAL_FILE_HEADER: u32 = 0x0201_4b50;
    const LOCAL_FILE_HEADER: u32 = 0x0403_4b50;

    /// Builds a minimal stored-member zip so the reader can be tested without
    /// depending on an external archiver.
    fn zip(members: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut directory = Vec::new();
        for (name, payload) in members {
            let offset = out.len() as u32;
            let crc = 0u32;
            out.extend_from_slice(&LOCAL_FILE_HEADER.to_le_bytes());
            out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // version, flags, method, time, date
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.write_all(payload).unwrap();

            directory.extend_from_slice(&CENTRAL_FILE_HEADER.to_le_bytes());
            directory.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            directory.extend_from_slice(&crc.to_le_bytes());
            directory.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            directory.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
            directory.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            directory.extend_from_slice(&offset.to_le_bytes());
            directory.extend_from_slice(name.as_bytes());
        }

        let directory_offset = out.len() as u32;
        let directory_size = directory.len() as u32;
        out.extend_from_slice(&directory);
        out.extend_from_slice(&END_OF_CENTRAL_DIRECTORY.to_le_bytes());
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(&(members.len() as u16).to_le_bytes());
        out.extend_from_slice(&(members.len() as u16).to_le_bytes());
        out.extend_from_slice(&directory_size.to_le_bytes());
        out.extend_from_slice(&directory_offset.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out
    }

    fn write_temp(bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "fabelgeist-numpy-storage-{}.npz",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn reads_stored_members() {
        let payload: Vec<u8> = [1.0f32, 2.0, 3.0, 4.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let archive = zip(&[
            (
                "weights.npy",
                crate::npy::tests::encode("<f4", "2, 2", &payload),
            ),
            (
                "indices.npy",
                crate::npy::tests::encode(
                    "<i8",
                    "2,",
                    &[7i64, 9]
                        .iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
            ),
        ]);

        let path = write_temp(&archive);
        let npz = Npz::open(&path).unwrap();
        assert_eq!(
            npz.keys().map(|name| name.to_string()).collect::<Vec<_>>(),
            ["weights", "indices"]
        );
        assert!(npz.get(&NpzArrayName::new("weights")).is_some());
        assert!(npz.get(&NpzArrayName::new("weights.npy")).is_some());
        assert!(npz.get(&NpzArrayName::new("missing")).is_none());

        let weights = npz.array(&NpzArrayName::new("weights")).unwrap();
        assert_eq!(
            weights.shape().dimensions(),
            [crate::NpyDimension::from(2); 2]
        );
        assert_eq!(
            Vec::<f32>::from(crate::NpyFloat32Values::from(&weights)),
            [1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(
            Vec::<i64>::from(crate::NpyIntegerValues::from(
                &npz.array(&NpzArrayName::new("indices")).unwrap()
            )),
            [7, 9]
        );

        let memory_npz = Npz::from_bytes(archive).unwrap();
        assert_eq!(
            Vec::<f32>::from(crate::NpyFloat32Values::from(
                &memory_npz.array(&NpzArrayName::new("weights")).unwrap()
            )),
            [1.0, 2.0, 3.0, 4.0]
        );

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn reports_a_missing_member() {
        let archive = zip(&[("a.npy", crate::npy::tests::encode("<f4", "1,", &[0; 4]))]);
        let path = write_temp(&archive);
        let npz = Npz::open(&path).unwrap();
        assert!(npz.array(&NpzArrayName::new("b")).is_err());
        std::fs::remove_file(path).ok();
    }
}
