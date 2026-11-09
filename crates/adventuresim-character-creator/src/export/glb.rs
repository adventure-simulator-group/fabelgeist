//! Checked binary glTF framing and source-preserving write failures.
//!
//! The complete frame is prepared before this writer creates directories or
//! opens the GLB file. Earlier export steps may already have published shared
//! images; a later filesystem write is not transactional.

use layout::GlbLayout;
use serde_json::Value;
use std::{fs, path::Path};

pub use error::{GlbError, GlbLayoutError, GlbLengthRole, GlbResult};

mod error;
mod layout;

pub(super) const GLB_MAGIC: &[u8; 4] = b"glTF";
const GLB_VERSION: u32 = 2;
const JSON_CHUNK: u32 = 0x4E4F_534A;
const BIN_CHUNK: u32 = 0x004E_4942;
const JSON_PADDING: u8 = b' ';
const BIN_PADDING: u8 = 0;

struct PreparedGlb(Vec<u8>);

impl PreparedGlb {
    fn new(document: &Value, bytes: Vec<u8>) -> GlbResult<Self> {
        let json_bytes =
            crate::profiling::measure("glb_serialize", || serde_json::to_vec(document))?;
        let layout = GlbLayout::new(json_bytes.len(), bytes.len())?;
        let mut glb = Vec::new();
        glb.try_reserve_exact(layout.byte_length())?;
        glb.extend_from_slice(GLB_MAGIC);
        glb.extend_from_slice(&GLB_VERSION.to_le_bytes());
        glb.extend_from_slice(&layout.encoded_length().to_le_bytes());
        glb.extend_from_slice(&layout.json_length().to_le_bytes());
        glb.extend_from_slice(&JSON_CHUNK.to_le_bytes());
        glb.extend_from_slice(&json_bytes);
        glb.extend(std::iter::repeat_n(JSON_PADDING, layout.json_padding()));
        glb.extend_from_slice(&layout.binary_length().to_le_bytes());
        glb.extend_from_slice(&BIN_CHUNK.to_le_bytes());
        glb.extend_from_slice(&bytes);
        glb.extend(std::iter::repeat_n(BIN_PADDING, layout.binary_padding()));
        Ok(Self(glb))
    }
}

pub(super) fn write(path: &Path, document: &Value, bytes: Vec<u8>) -> GlbResult<()> {
    let packet = PreparedGlb::new(document, bytes)?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| GlbError::CreateDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    crate::profiling::measure("glb_write", || fs::write(path, packet.0)).map_err(|source| {
        GlbError::Write {
            path: path.to_path_buf(),
            source,
        }
    })
}
