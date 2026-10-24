use crate::prelude::*;

pub(super) struct CompiledLayout {
    pub(super) layout: wgpu::BindGroupLayout,
    pub(super) pipeline: wgpu::ComputePipeline,
}

impl CompiledLayout {
    pub(super) fn new(
        context: &WgpuContext,
        entries: &[wgpu::BindGroupLayoutEntry],
        module: &wgpu::ShaderModule,
        entry_point: &str,
    ) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = ValidationScope::new(context);
        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Kernel Bind Layout"),
                entries,
            });
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Kernel Pipeline Layout"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Kernel Pipeline"),
                layout: Some(&pipeline_layout),
                module,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            error_scope
                .finish()
                .map_err(fabelgeist_gpu::data::ComputePipelineError::from)?;
        }
        Ok(Self { layout, pipeline })
    }
}
