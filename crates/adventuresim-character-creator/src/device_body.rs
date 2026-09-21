//! A body realization's upload to the armor device, made once.

use std::sync::OnceLock;

use adventuresim_armor_model::gpu::body::GpuBody;
use anyhow::Result;

/// The device copy of one body realization -- the wearer or a morph sample.
///
/// Every armor piece fitted to a body reads the same positions, normals and
/// skin; the realization keeps its upload so that the pieces share it rather
/// than each sending the body, and every morph sample, again.
#[derive(Clone, Debug, Default)]
pub struct DeviceBody(OnceLock<GpuBody>);

impl DeviceBody {
    /// The uploaded body, uploading it with `upload` on first use.
    pub fn get_or_upload(&self, upload: impl FnOnce() -> Result<GpuBody>) -> Result<&GpuBody> {
        if let Some(body) = self.0.get() {
            return Ok(body);
        }
        let body = upload()?;
        // A racing thread may have uploaded it too; either copy is the same.
        Ok(self.0.get_or_init(|| body))
    }
}
