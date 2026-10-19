//! Versioned little-endian rendering payloads with checked length boundaries.

use bytemuck::Pod;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use super::meshes::GridCardinality;
use super::{AddressMesh, LibraryId, PreparedError, Result};

#[cfg(target_family = "wasm")]
mod browser_identity;
mod headers;
mod library;
mod population;
mod unique;

#[cfg(target_family = "wasm")]
pub use browser_identity::BrowserIdentityFailure;
pub use library::{decode_library, encode_library};
pub use population::{decode_population, encode_population};
pub use unique::{decode_unique, encode_unique};

const MAX_METADATA_BYTES: usize = 4 * 1024 * 1024;
const UNIQUE_MAGIC: &[u8; 8] = b"FORESTU1";
const LIBRARY_MAGIC: &[u8; 8] = b"FORESTL2";
const POPULATION_MAGIC: &[u8; 8] = b"FORESTP2";

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let (value, rest) = self
            .0
            .split_at_checked(length)
            .ok_or(PreparedError::Invalid("truncated payload"))?;
        self.0 = rest;
        Ok(value)
    }
    fn count(&mut self) -> Result<usize> {
        let word: [u8; 8] = self
            .take(8)?
            .try_into()
            .map_err(|_| PreparedError::Invalid("length word"))?;
        usize::try_from(u64::from_le_bytes(word))
            .map_err(|_| PreparedError::Invalid("length overflow"))
    }
    fn header<T: DeserializeOwned>(&mut self, magic: &[u8; 8]) -> Result<T> {
        if !cfg!(target_endian = "little") || self.take(8)? != magic {
            return Err(PreparedError::Invalid(
                "unsupported payload format or byte order",
            ));
        }
        let count = self.count()?;
        if count > MAX_METADATA_BYTES {
            return Err(PreparedError::Invalid("metadata size limit"));
        }
        Ok(serde_json::from_slice(self.take(count)?)?)
    }
    fn records<T: Pod>(&mut self) -> Result<Vec<T>> {
        let count = self.count()?;
        self.array(count)
    }
    fn array<T: Pod>(&mut self, count: usize) -> Result<Vec<T>> {
        let length = count
            .checked_mul(size_of::<T>())
            .ok_or(PreparedError::Invalid("record byte length overflow"))?;
        Ok(self
            .take(length)?
            .chunks_exact(size_of::<T>())
            .map(bytemuck::pod_read_unaligned)
            .collect())
    }
    fn exact_records<T: Pod>(&mut self, expected: u32) -> Result<Vec<T>> {
        if self.count()? as u64 != u64::from(expected) {
            return Err(PreparedError::MeshCardinality);
        }
        self.array(expected as usize)
    }
    fn mesh(&mut self, expected: GridCardinality) -> Result<AddressMesh> {
        Ok(AddressMesh {
            vertices: self.exact_records(expected.vertices)?,
            indices: self.exact_records(expected.indices)?,
        })
    }
    fn finish(self) -> Result<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(PreparedError::Invalid("trailing payload"))
        }
    }
}

fn begin(magic: &[u8; 8], header: &impl Serialize) -> Result<Vec<u8>> {
    if !cfg!(target_endian = "little") {
        return Err(PreparedError::Invalid("unsupported byte order"));
    }
    let metadata = serde_json::to_vec(header)?;
    if metadata.len() > MAX_METADATA_BYTES {
        return Err(PreparedError::Invalid("metadata size limit"));
    }
    let mut bytes = magic.to_vec();
    bytes.extend_from_slice(&(metadata.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&metadata);
    Ok(bytes)
}

fn records<T: Pod>(bytes: &mut Vec<u8>, values: &[T]) {
    bytes.extend_from_slice(&(values.len() as u64).to_le_bytes());
    bytes.extend_from_slice(bytemuck::cast_slice(values));
}

fn mesh(bytes: &mut Vec<u8>, mesh: &AddressMesh) {
    records(bytes, &mesh.vertices);
    records(bytes, &mesh.indices);
}

fn identity(bytes: &[u8]) -> LibraryId {
    LibraryId(Sha256::digest(bytes).into())
}
