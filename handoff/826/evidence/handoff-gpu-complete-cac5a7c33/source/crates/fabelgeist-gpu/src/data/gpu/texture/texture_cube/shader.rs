//! Cube shader admission, native resources and observed command recording.

use super::TextureCube;
use crate::data::ComputePassError;
use crate::data::gpu::texture::{CubeFace, TextureFormat};
use crate::globals::WgpuContext;
use anyhow::Result;
use std::sync::Arc;
use wgpu::util::DeviceExt;

mod source;

/// A native group and the exact layout used to construct it.
struct CubeShaderBindings {
    layout: wgpu::BindGroupLayout,
    group: wgpu::BindGroup,
}

impl TextureCube {
    pub fn render_face_shader(
        context: &WgpuContext,
        target: TextureCube,
        face: CubeFace,
        shader_src: String,
        time: f32,
    ) -> Result<TextureCube> {
        target.render_face_shader_raw(context, face, &shader_src, time)?;
        Ok(target)
    }

    pub fn render_face_shader_raw(
        &self,
        context: &WgpuContext,
        face: CubeFace,
        shader_src: &str,
        time: f32,
    ) -> Result<()> {
        let view = self.view_2d_array(context)?;
        let format = self.effective_format(self.format);
        let code = source::face(shader_src, format.to_wgsl_storage_format()?);
        let uniform_bytes = [
            time.to_ne_bytes(),
            face.index().to_ne_bytes(),
            0.0f32.to_ne_bytes(),
            0.0f32.to_ne_bytes(),
        ]
        .concat();
        let encoder = self
            .record_shader(context, &view, code, &uniform_bytes, 1)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        Ok(())
    }

    pub fn render_shader(
        context: &WgpuContext,
        target: TextureCube,
        shader_src: String,
        time: f32,
    ) -> Result<TextureCube> {
        target.render_shader_raw(context, &shader_src, time)?;
        Ok(target)
    }

    pub fn render_shader_raw(
        &self,
        context: &WgpuContext,
        shader_src: &str,
        time: f32,
    ) -> Result<()> {
        let view = self.view_2d_array(context)?;
        let format = self.effective_format(self.format);
        let code = source::all_faces(shader_src, format.to_wgsl_storage_format()?);
        let uniforms = [time, 0.0, 0.0, 0.0];
        let encoder = self
            .record_shader(
                context,
                &view,
                code,
                bytemuck::cast_slice(&uniforms),
                CubeFace::ALL.len() as u32,
            )
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        Ok(())
    }

    /// Observe construction and recording before the separate submit scopes.
    /// This immediate observation adds no device poll or completion promise.
    fn record_shader(
        &self,
        context: &WgpuContext,
        view: &wgpu::TextureView,
        code: String,
        uniforms: &[u8],
        layers: u32,
    ) -> std::result::Result<wgpu::CommandEncoder, wgpu::Error> {
        context.observe_validation(|| {
            let shader = context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("TextureCube Shader Renderer"),
                    source: wgpu::ShaderSource::Wgsl(code.into()),
                });
            let bindings = CubeShaderBindings::new(
                context,
                view,
                self.effective_format(self.format),
                uniforms,
            );
            let layout = context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("TextureCube Renderer Pipeline Layout"),
                    bind_group_layouts: &[Some(&bindings.layout)],
                    immediate_size: 0,
                });
            let pipeline =
                context
                    .device
                    .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("TextureCube Renderer Compute Pipeline"),
                        layout: Some(&layout),
                        module: &shader,
                        entry_point: Some("main"),
                        compilation_options: Default::default(),
                        cache: None,
                    });
            let mut encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("TextureCube Renderer Command Encoder"),
                    });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("TextureCube Renderer Compute Pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bindings.group, &[]);
                let workgroups = self.size.div_ceil(8);
                pass.dispatch_workgroups(workgroups, workgroups, layers);
            }
            encoder
        })
    }
}

impl CubeShaderBindings {
    fn new(
        context: &WgpuContext,
        view: &wgpu::TextureView,
        format: TextureFormat,
        uniforms: &[u8],
    ) -> Self {
        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("TextureCube Renderer Uniforms"),
                contents: uniforms,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("TextureCube Renderer Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: format.into(),
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TextureCube Renderer Bind Group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            });
        Self { layout, group }
    }
}
