pub use error::*;
pub use grid::*;
pub use shape::*;

use crate::data::{ComputePipeline, PassParameters};
use crate::globals::WgpuContext;
use anyhow::Result;

mod error;
mod grid;
mod shape;

pub struct ComputePass;

impl ComputePass {
    /// Record and submit a standalone dispatch under the context's validation
    /// policy. Successful submission does not establish GPU completion.
    pub fn dispatch(
        context: &WgpuContext,
        pipeline_def: ComputePipeline,
        parameters: PassParameters,
        groups: WorkgroupGrid,
    ) -> Result<()> {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ComputePass Encoder"),
            });

        Self::record(context, &pipeline_def, &parameters, &mut encoder, groups)?;

        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(std::sync::Arc::new(cause)))?;
        Ok(())
    }

    /// Append a dispatch after admitting its pipeline and bindings. Native
    /// encoding errors can remain deferred until the encoder is finalized.
    pub fn record(
        context: &WgpuContext,
        pipeline_def: &ComputePipeline,
        parameters: &PassParameters,
        encoder: &mut wgpu::CommandEncoder,
        groups: WorkgroupGrid,
    ) -> Result<()> {
        let groups = groups.validate_device(&context.device)?;
        let pipeline = pipeline_def.compiled(context)?;
        let bind_groups =
            crate::data::gpu::bind_group::create_bind_groups(context, pipeline_def, parameters)?;

        #[cfg(not(target_arch = "wasm32"))]
        let scope = context.blocking_validation.then(|| {
            context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });
        // Frame: Compute Pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ComputePass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline);
            for bound in &bind_groups {
                bound.bind(&mut compute_pass);
            }
            groups.record(&mut compute_pass);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ComputePassError::Native(std::sync::Arc::new(cause)).into());
            }
        }

        Ok(())
    }
}
