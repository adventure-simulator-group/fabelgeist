//! Encode and write the binary glTF container.

use anyhow::{Context, Result};
use serde_json::Value;
use std::{fs, path::Path};

pub(super) const GLB_MAGIC: &[u8; 4] = b"glTF";
const GLB_VERSION: u32 = 2;
const JSON_CHUNK: u32 = 0x4E4F_534A;
const BIN_CHUNK: u32 = 0x004E_4942;

pub(super) fn write(path: &Path, document: &Value, mut bytes: Vec<u8>) -> Result<()> {
    let mut json_bytes = serde_json::to_vec(&document)?;
    json_bytes.resize(json_bytes.len().next_multiple_of(4), b' ');
    bytes.resize(bytes.len().next_multiple_of(4), 0);
    let total_length = 12 + 8 + json_bytes.len() + 8 + bytes.len();
    let mut glb = Vec::with_capacity(total_length);
    glb.extend_from_slice(GLB_MAGIC);
    glb.extend_from_slice(&GLB_VERSION.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&JSON_CHUNK.to_le_bytes());
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(&BIN_CHUNK.to_le_bytes());
    glb.extend_from_slice(&bytes);

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating export directory {}", parent.display()))?;
    }
    fs::write(path, glb).with_context(|| format!("writing {}", path.display()))
}
