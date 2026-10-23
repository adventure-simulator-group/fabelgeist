//! Explicit compute entry selection, including source-order first selection.

use super::{ComputePipelineError, ComputePipelineResult};

/// Which compute entry a pipeline definition selects from its shader.
#[derive(Clone, Copy, Debug)]
pub enum ComputeEntryPoint<'a> {
    /// Preserve source order for callers choosing the first compute program.
    First,
    /// Select this exact compute entry; absence is an error, with no fallback.
    Named(&'a str),
}

impl ComputeEntryPoint<'_> {
    /// Resolve an identity in an already parsed native shader module.
    ///
    /// This does not validate the module or compile a pipeline. Callers can
    /// inspect the selected entry's workgroup shape before native creation.
    pub fn resolve(
        self,
        module: &wgpu::naga::Module,
    ) -> ComputePipelineResult<&wgpu::naga::EntryPoint> {
        module
            .entry_points
            .iter()
            .find(|entry| {
                entry.stage == wgpu::naga::ShaderStage::Compute
                    && match self {
                        Self::First => true,
                        Self::Named(name) => entry.name == name,
                    }
            })
            .ok_or_else(|| ComputePipelineError::MissingEntryPoint {
                requested: match self {
                    Self::First => None,
                    Self::Named(name) => Some(name.to_owned()),
                },
            })
    }
}
