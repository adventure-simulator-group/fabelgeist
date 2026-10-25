//! Reduction recording and device-associated reusable scratch buffers.
//!
//! Each level has exactly its live byte extent: the shader's arrayLength then
//! describes logical input, including when later calls reuse the allocation.

use super::layout::ReductionElements;
use super::{Reduce, ReduceDefinition, ReductionPipeline};
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::prelude::{BufferElementStride, BufferUse};

#[derive(Default)]
pub struct ReduceScratchpad {
    buffers: Option<ReductionBuffers>,
}

struct ReductionBuffers {
    device: wgpu::Device,
    levels: Vec<Buffer>,
}

struct ReductionPlan {
    first: ReductionPipeline,
    recurring: ReductionPipeline,
    input_count: ReductionElements,
}

impl Reduce {
    pub fn execute(
        context: &WgpuContext,
        definition: &ReduceDefinition,
        input: &GpuResource,
        scratchpad: &mut ReduceScratchpad,
    ) -> Result<Buffer> {
        let plan = ReductionPlan::new(context, definition, input)?;
        if plan.input_count.is_finished() {
            return match input {
                GpuResource::Buffer(buffer) => Ok(buffer.clone()),
                _ => Err(anyhow!("Cannot reduce texture of size 1 to buffer")),
            };
        }

        let buffers = scratchpad.prepare(context, plan.input_count, plan.first.stride)?;
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Reduce Batch Encoder"),
            });
        let reduced = plan.record(context, input, buffers, &mut encoder)?;
        let bytes = plan.first.stride.byte_length();
        let result = Buffer::new(
            context,
            bytes,
            BufferDefinition::storage()
                .with_label("reduction_result".into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;
        encoder.copy_buffer_to_buffer(&reduced.buffer, 0, &result.buffer, 0, u64::from(bytes));
        context.queue.submit(std::iter::once(encoder.finish()));
        Ok(result)
    }
}

impl ReduceScratchpad {
    fn prepare(
        &mut self,
        context: &WgpuContext,
        input_count: ReductionElements,
        stride: BufferElementStride,
    ) -> Result<&ReductionBuffers> {
        let previous = self
            .buffers
            .as_ref()
            .filter(|buffers| &buffers.device == context.device.as_ref());
        let mut levels = Vec::new();
        let mut output_count = input_count.reduced();
        loop {
            let previous_level = previous.and_then(|buffers| buffers.levels.get(levels.len()));
            levels.push(scratch_buffer(
                context,
                previous_level,
                output_count.byte_length(stride),
            )?);
            if output_count.is_finished() {
                break;
            }
            output_count = output_count.reduced();
        }
        Ok(self.buffers.insert(ReductionBuffers {
            device: context.device.as_ref().clone(),
            levels,
        }))
    }
}

impl ReductionPlan {
    fn new(context: &WgpuContext, definition: &ReduceDefinition, input: &GpuResource) -> Result<Self> {
        let ty = fabelgeist_gpu::data::gpu::signature::parse_binary_op_signature(
            &definition.code,
            "reduce",
        )?;
        let first = definition.get_or_create_pipeline(
            context,
            ResourceDescriptor::from_resource(input, ty.clone()),
        )?;
        let recurring = definition.get_or_create_pipeline(context, ResourceDescriptor::Buffer(ty))?;
        let input_count = ReductionElements::from_resource(input, first.stride)?;
        Ok(Self {
            first,
            recurring,
            input_count,
        })
    }

    fn record(
        &self,
        context: &WgpuContext,
        input: &GpuResource,
        buffers: &ReductionBuffers,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Buffer> {
        let mut current_resource = input.clone();
        let mut current_count = self.input_count;
        for (pass_index, output) in buffers.levels.iter().enumerate() {
            let mut parameters = PassParameters::new();
            match &current_resource {
                GpuResource::Buffer(buffer) => parameters.insert("input", buffer.clone()),
                GpuResource::Texture2d(texture) => parameters.insert("input", texture.clone()),
                GpuResource::Texture3d(texture) => parameters.insert("input", texture.clone()),
            }
            parameters.insert("output", output.clone());
            let pipeline = if pass_index == 0 {
                &self.first.pipeline
            } else {
                &self.recurring.pipeline
            };
            ComputePass::record(
                context,
                pipeline,
                &parameters,
                encoder,
                current_count.covering_workgroups(),
            )?;
            current_resource = GpuResource::Buffer(output.clone());
            current_count = current_count.reduced();
        }
        match current_resource {
            GpuResource::Buffer(buffer) => Ok(buffer),
            _ => Err(anyhow!("Reduction failed to produce output")),
        }
    }
}

fn scratch_buffer(
    context: &WgpuContext,
    previous: Option<&Buffer>,
    bytes: BufferByteLength,
) -> Result<Buffer> {
    if let Some(buffer) = previous
        && buffer.size == bytes
    {
        return Ok(buffer.clone());
    }
    Buffer::new(
        context,
        bytes,
        BufferDefinition::storage()
            .with_label("scratchpad".into())
            .with_usage(BufferUse::CopySource),
    )
}
