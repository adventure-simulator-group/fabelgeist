//! Native package hashing boundaries shared by terrain and vector roads.
use crate::Result;
use adventuresim_world_schema::source_package::SourcePackageDigest;
use sha2::{Digest, Sha256};
use std::io::Read;

const HASH_READ_BUFFER_BYTES: usize = 1024 * 1024;

pub(crate) fn valid_digest(value: &str) -> bool {
    SourcePackageDigest::from_hex(value).is_ok()
}

pub(crate) fn hex_sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn hex_sha_reader(mut reader: impl Read) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; HASH_READ_BUFFER_BYTES];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
