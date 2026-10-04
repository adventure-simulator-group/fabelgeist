use crate::data::PassParameters;
use crate::data::gpu::compute::pipeline::ComputePipeline;
use crate::globals::WgpuContext;

mod bindings;
mod error;
mod grid;
pub use grid::{DispatchOccupancy, WorkgroupGrid};
mod shape;
pub use shape::{InvocationCount, WorkgroupShape, WorkgroupShapeError};
mod texture;
pub use error::{BindingKind, ComputePassError};

pub struct ComputePass;
impl ComputePass {
    pub fn dispatch(
        context: &WgpuContext,
        pipeline: ComputePipeline,
        parameters: PassParameters,
        groups: WorkgroupGrid,
    ) -> Result<(), ComputePassError> {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ComputePass Encoder"),
            });
        Self::record(context, &pipeline, &parameters, &mut encoder, groups)?;
        context.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
    pub fn record(
        context: &WgpuContext,
        pipeline: &ComputePipeline,
        parameters: &PassParameters,
        encoder: &mut wgpu::CommandEncoder,
        groups: WorkgroupGrid,
    ) -> Result<(), ComputePassError> {
        let native = pipeline
            .pipeline
            .as_ref()
            .ok_or(ComputePassError::MissingPipeline)?;
        if let Ok(guard) = pipeline.validation_error.lock()
            && let Some(cause) = guard.as_ref()
        {
            return Err(ComputePassError::InvalidPipeline(cause.clone()));
        }
        let reflection = pipeline
            .reflection
            .as_ref()
            .ok_or(ComputePassError::MissingReflection)?;
        let mut bind_groups = Vec::new();
        for group in &reflection.bind_groups {
            let layout = group
                .index
                .layout(&pipeline.bind_group_layouts)
                .ok_or(ComputePassError::MissingLayout(group.index))?;
            bind_groups.push(bindings::BoundGroup::new(
                context, group, layout, parameters,
            )?);
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ComputePass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(native);
        for (index, group) in bind_groups.iter().enumerate() {
            pass.set_bind_group(index as u32, &group.native, &[]);
        }
        groups.record(&mut pass);
        Ok(())
    }
}
