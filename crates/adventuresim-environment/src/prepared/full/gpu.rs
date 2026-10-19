//! Owned compute submission and bounded whole-record readback scratch.

use anyhow::{Context, Result, ensure};
use bytemuck::{Pod, Zeroable};
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferDefinition, BufferUpload, BufferUse, WgpuContext,
};

use super::{plan::PartPlan, shader::Kernel};
use crate::{prepared::TreeLibrary, render::ForestPart};

const _: () = {
    assert!(size_of::<ExpandedVertex>() == 48);
    assert!(std::mem::offset_of!(ExpandedVertex, normal) == 16);
    assert!(std::mem::offset_of!(ExpandedVertex, uv) == 32);
    assert!(std::mem::offset_of!(ExpandedVertex, address) == 40);
    assert!(std::mem::offset_of!(ExpandedVertex, pad) == 44);
    assert!(size_of::<Params>() == 32);
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ExpandedVertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub uv: [f32; 2],
    pub address: u32,
    pub pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Params {
    pub(super) shape: [u32; 4],
    pub(super) span: [u32; 4],
}

pub(super) struct NativeExpansion {
    context: WgpuContext,
    branch: Kernel,
    leaf: Kernel,
}

impl NativeExpansion {
    pub async fn new() -> Result<Self> {
        let context = WgpuContext::new_compute().await?;
        let limits = context.device.limits();
        ensure!(
            limits.max_bind_groups >= 1
                && limits.max_bindings_per_bind_group >= 3
                && limits.max_storage_buffers_per_shader_stage >= 2
                && limits.max_uniform_buffers_per_shader_stage >= 1
                && limits.max_uniform_buffer_binding_size as usize >= size_of::<Params>(),
            "device cannot bind the Full authoring ABI"
        );
        let branch = Kernel::new(&context, ForestPart::Branches)?;
        let leaf = Kernel::new(&context, ForestPart::Leaves)?;
        Ok(Self {
            context,
            branch,
            leaf,
        })
    }

    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.context.adapter.get_info()
    }

    pub fn record_capacity(
        &self,
        role: ForestPart,
        vertices_per_record: u32,
        record_stride: u64,
        vertex_limit: u32,
    ) -> Result<u32> {
        ensure!(
            vertices_per_record > 0 && record_stride > 0,
            "invalid Full record shape"
        );
        let limits = self.context.device.limits();
        let bytes = limits
            .max_buffer_size
            .min(limits.max_storage_buffer_binding_size);
        let width = u64::from(self.kernel(role).width.get());
        let dispatch_vertices = u64::from(limits.max_compute_workgroups_per_dimension) * width;
        let per_record = u64::from(vertices_per_record);
        let output_stride = size_of::<ExpandedVertex>() as u64;
        let cap = (u64::from(vertex_limit) / per_record)
            .min(bytes / record_stride)
            .min(bytes / (per_record * output_stride))
            .min(dispatch_vertices / per_record)
            .min(u64::from(u32::MAX));
        Ok(u32::try_from(cap)?)
    }

    pub async fn expand(
        &self,
        library: &TreeLibrary,
        part: &PartPlan,
    ) -> Result<Vec<ExpandedVertex>> {
        match part.geometry.role() {
            ForestPart::Branches => {
                let records = library
                    .branches
                    .get(part.source_range()?)
                    .context("Full wood span outside bank")?;
                self.dispatch(records, part).await
            }
            ForestPart::Leaves => {
                let records = library
                    .leaves
                    .get(part.source_range()?)
                    .context("Full foliage span outside bank")?;
                self.dispatch(records, part).await
            }
        }
    }

    fn kernel(&self, role: ForestPart) -> &Kernel {
        match role {
            ForestPart::Branches => &self.branch,
            ForestPart::Leaves => &self.leaf,
        }
    }

    async fn dispatch<T: Pod>(
        &self,
        records: &[T],
        part: &PartPlan,
    ) -> Result<Vec<ExpandedVertex>> {
        ensure!(
            records.len() == usize::try_from(part.records.count)?,
            "Full partition input count mismatch"
        );
        let context = &self.context;
        let kernel = self.kernel(part.geometry.role());
        let upload = BufferUpload::from_elements(records);
        let input = Buffer::new(context, upload.length(), BufferDefinition::storage())?;
        input.write(context, upload)?;
        let params = Params {
            shape: part.geometry.shader_words()?,
            span: [part.records.start, part.records.count, part.vertices, 0],
        };
        let upload = BufferUpload::from_elements(std::slice::from_ref(&params));
        let uniform = Buffer::new(
            context,
            upload.length(),
            BufferDefinition::uniform().with_usage(BufferUse::CopyDestination),
        )?;
        uniform.write(context, upload)?;
        let output_bytes = u64::from(part.vertices)
            .checked_mul(size_of::<ExpandedVertex>() as u64)
            .context("Full output byte overflow")?;
        let output = Buffer::new(
            context,
            BufferByteLength::from(output_bytes),
            BufferDefinition::storage(),
        )?;
        let scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let result = self
            .submit_and_read(part, kernel, &input, &uniform, &output)
            .await;
        if let Some(error) = scope.pop().await {
            anyhow::bail!("Full compute validation: {error}");
        }
        result
    }

    async fn submit_and_read(
        &self,
        part: &PartPlan,
        kernel: &Kernel,
        input: &Buffer,
        uniform: &Buffer,
        output: &Buffer,
    ) -> Result<Vec<ExpandedVertex>> {
        let context = &self.context;
        let layout = kernel
            .pipeline
            .bind_group_layouts
            .first()
            .context("missing Full group0")?;
        let group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("grown Full expansion"),
                layout: layout.as_ref(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: output.buffer.as_entire_binding(),
                    },
                ],
            });
        let pipeline = kernel
            .pipeline
            .pipeline
            .as_deref()
            .context("uncompiled Full kernel")?;
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("grown Full expansion"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("grown Full expansion"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(part.vertices.div_ceil(kernel.width.get()), 1, 1);
        }
        context.queue.submit(Some(encoder.finish()));
        output.read::<ExpandedVertex>(context).await
    }
}
