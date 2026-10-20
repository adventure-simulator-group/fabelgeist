//! Same-frame prepared source allocations, kept with the successful submission.
use super::*;
use adventuresim_environment::render::grown::GrownBufferRole;
use bevy::render::{
    render_asset::RenderAssets,
    render_resource::BufferId,
    storage::{GpuShaderBuffer, ShaderBuffer},
};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PreparedSource {
    role: GrownBufferRole,
    asset: AssetId<ShaderBuffer>,
    buffer: BufferId,
    bytes: u64,
}

impl PreparedSource {
    pub(super) fn resolve(
        bindings: &[inventory::SourceBinding],
        prepared: &RenderAssets<GpuShaderBuffer>,
    ) -> Result<Option<Vec<Self>>> {
        let prepared: Option<Vec<Self>> = bindings
            .iter()
            .map(|binding| {
                let actual = prepared.get(binding.handle.id())?;
                Some(Self {
                    role: binding.role,
                    asset: binding.handle.id(),
                    buffer: actual.buffer.id(),
                    bytes: actual.buffer.size(),
                })
            })
            .collect();
        let Some(prepared) = prepared else {
            return Ok(None);
        };
        ensure!(
            prepared
                .iter()
                .enumerate()
                .all(|(index, source)| prepared[..index]
                    .iter()
                    .all(|earlier| earlier.buffer != source.buffer)),
            "grown GPU source roles alias an allocation"
        );
        Ok(Some(prepared))
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"role":format!("{:?}",self.role),"asset":format!("{:?}",self.asset),
            "buffer":format!("{:?}",self.buffer),"logical_allocation_bytes":self.bytes,
            "limits":"prepared native allocation identity and size, not storage bytes or physical memory"})
    }
}
