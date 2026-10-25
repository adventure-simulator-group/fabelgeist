//! Native cubemap face copies and sampled blits.

use super::TextureCube;
use crate::data::ComputePassError;
use crate::data::gpu::texture::{CubeFace, TextureFormat, TextureView, TextureViewError};
use crate::globals::WgpuContext;
use anyhow::Result;
use std::sync::Arc;

const BLIT_SOURCE: &str = r#"
            struct VertexOutput {
                @builtin(position) position: vec4<f32>,
                @location(0) uv: vec2<f32>,
            };

            @vertex
            fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
                var out: VertexOutput;
                let x = f32(i32(idx & 1u) * 4 - 1);
                let y = f32(i32(idx & 2u) * 2 - 1);
                out.position = vec4<f32>(x, y, 0.0, 1.0);
                out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
                return out;
            }

            @group(0) @binding(0) var src_tex: texture_2d<f32>;
            @group(0) @binding(1) var src_samp: sampler;

            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return textureSample(src_tex, src_samp, in.uv);
            }
        "#;

struct CubeBlitBindings {
    layout: wgpu::BindGroupLayout,
    group: wgpu::BindGroup,
}

impl TextureCube {
    pub fn set_side_texture(
        &self,
        context: &WgpuContext,
        face: CubeFace,
        texture: &TextureView,
    ) -> Result<()> {
        let dst_texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;
        self.ensure_device(context)?;

        if let Some(src_texture) = &texture.texture
            && texture.size.0 == self.size
            && texture.size.1 == self.size
            && texture.format == self.format
            && texture.layer.is_none()
        {
            texture.ensure_device(context)?;
            let encoder = context
                .observe_validation(|| {
                    let mut encoder =
                        context
                            .device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("TextureCube Set Side Copy Encoder"),
                            });
                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: src_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyTextureInfo {
                            texture: dst_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: face.index(),
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::Extent3d {
                            width: self.size,
                            height: self.size,
                            depth_or_array_layers: 1,
                        },
                    );
                    encoder
                })
                .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
            context
                .submit(encoder)
                .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
            return Ok(());
        }

        self.blit_texture_to_face(context, face, texture)
    }
    pub fn blit_texture_to_face(
        &self,
        context: &WgpuContext,
        face: CubeFace,
        texture: &TextureView,
    ) -> Result<()> {
        let source = texture.selected_view(context)?.view;
        let target = self.face_view_with_format(Some(context), face, self.format)?;
        let target = target.selected_view(context)?.view;
        let encoder = context
            .observe_validation(|| self.record_blit(context, &source, &target))
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        Ok(())
    }

    fn record_blit(
        &self,
        context: &WgpuContext,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
    ) -> wgpu::CommandEncoder {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("TextureCube Face Blit Shader"),
                source: wgpu::ShaderSource::Wgsl(BLIT_SOURCE.into()),
            });
        let bindings = CubeBlitBindings::new(context, source);
        let pipeline =
            bindings.create_pipeline(context, &shader, self.effective_format(self.format));
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("TextureCube Blit Command Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("TextureCube Blit Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bindings.group, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder
    }
}

impl CubeBlitBindings {
    fn new(context: &WgpuContext, view: &wgpu::TextureView) -> Self {
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TextureCube Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("TextureCube Blit Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TextureCube Blit Bind Group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
        Self { layout, group }
    }

    fn create_pipeline(
        &self,
        context: &WgpuContext,
        shader: &wgpu::ShaderModule,
        format: TextureFormat,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("TextureCube Blit Pipeline Layout"),
                    bind_group_layouts: &[Some(&self.layout)],
                    immediate_size: 0,
                });

        context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("TextureCube Blit Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: format.into(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
    }
}
