use std::sync::Arc;

mod image_data;

use anyhow::Result;

use crate::{
    data::gpu::texture::{CubeFace, Image, TextureFormat, TextureView},
    globals::WgpuContext,
};

#[derive(Clone, Debug)]
pub struct TextureCube {
    pub texture: Option<Arc<wgpu::Texture>>,
    pub view: Option<Arc<wgpu::TextureView>>,
    pub size: u32,
    pub format: TextureFormat,
    pub usage: wgpu::TextureUsages,
}

impl PartialEq for TextureCube {
    fn eq(&self, other: &Self) -> bool {
        (match (&self.texture, &other.texture) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }) && (match (&self.view, &other.view) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }) && self.size == other.size
            && self.format == other.format
            && self.usage == other.usage
    }
}

impl Default for TextureCube {
    fn default() -> Self {
        Self {
            texture: None,
            view: None,
            size: 0,
            format: TextureFormat::default(),
            usage: wgpu::TextureUsages::empty(),
        }
    }
}
impl TextureCube {
    pub fn new(context: &WgpuContext, size: f32, format: TextureFormat) -> Result<TextureCube> {
        let _wgpu_format: wgpu::TextureFormat = format.into();

        if size.is_nan() {
            return Err(anyhow::anyhow!("Texture size contains NaN: {:?}", size));
        }

        let size = size as u32;

        if size < 1 {
            return Err(anyhow::anyhow!(
                "Texture size must be at least 1. Got {} (from {:?})",
                size,
                size
            ));
        }

        let limits = context.device.limits();
        let max_dim = limits.max_texture_dimension_2d;
        if size > max_dim {
            return Err(anyhow::anyhow!(
                "Texture size {} exceeds maximum supported dimension {} (from {:?})",
                size,
                max_dim,
                size
            ));
        }

        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC;

        if format.supports_render_attachment() {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }

        if format.supports_storage() && !format.is_srgb() {
            usage |= wgpu::TextureUsages::STORAGE_BINDING;
        }

        let base_format = format.linear_counterpart();
        let wgpu_base_format: wgpu::TextureFormat = base_format.into();

        let mut view_formats = vec![wgpu_base_format];
        let counterpart = base_format.srgb_counterpart().into();
        if counterpart != wgpu_base_format {
            view_formats.push(counterpart);
        }

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("TextureCube"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_base_format,
            usage,
            view_formats: &view_formats,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let texture = context.device.create_texture(&texture_desc);

        let mut initial_view_format = format;
        if usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            initial_view_format = format.linear_counterpart();
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("TextureCube View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            format: Some(initial_view_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(err) = pollster::block_on(error_scope.pop()) {
                return Err(anyhow::anyhow!("WGPU TextureCube Creation Error: {}", err));
            }
        }

        let texture_value = TextureCube {
            texture: Some(Arc::new(texture)),
            view: Some(Arc::new(view)),
            size,
            format,
            usage,
        };

        Ok(texture_value)
    }

    /// A cube map holding these six images' pixels.
    ///
    /// Each face is stored as it arrived, on the same terms as a 2D texture:
    /// see [`Image::packed_for`].
    pub fn create_from_images(
        context: &WgpuContext,
        images: [Image; 6],
        format: TextureFormat,
    ) -> Result<TextureCube> {
        let size = images[0].width;
        for (i, img) in images.iter().enumerate() {
            if img.width != size || img.height != size {
                return Err(anyhow::anyhow!(
                    "Cube map images must be square and identical in size. Face 0: {}x{}, Face {}: {}x{}",
                    size,
                    size,
                    i,
                    img.width,
                    img.height
                ));
            }
        }

        let tex = TextureCube::new(context, size as f32, format)?;
        let pixel_size = format.pixel_size();

        for (i, image) in images.into_iter().enumerate() {
            let converted_data = image.packed_for(format)?;

            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: tex.texture.as_ref().unwrap(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &converted_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size * pixel_size),
                    rows_per_image: Some(size),
                },
                wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
            );
        }

        Ok(tex)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the cubemap boundary names its six independently optional faces"
    )]
    pub fn from_textures(
        context: &WgpuContext,
        right: Option<TextureView>,
        left: Option<TextureView>,
        top: Option<TextureView>,
        bottom: Option<TextureView>,
        front: Option<TextureView>,
        back: Option<TextureView>,
        format: Option<TextureFormat>,
        size: Option<f64>,
    ) -> Result<TextureCube> {
        let size_val = size
            .map(|s| s as f32)
            .or_else(|| right.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| left.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| top.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| bottom.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| front.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| back.as_ref().map(|t| t.size.0 as f32))
            .unwrap_or(256.0);

        let format_val = format
            .or_else(|| right.as_ref().map(|t| t.format))
            .unwrap_or(TextureFormat::Rgba8UnormSrgb);

        let cube = TextureCube::new(context, size_val, format_val)?;

        let faces = [
            (CubeFace::PositiveX, right),
            (CubeFace::NegativeX, left),
            (CubeFace::PositiveY, top),
            (CubeFace::NegativeY, bottom),
            (CubeFace::PositiveZ, front),
            (CubeFace::NegativeZ, back),
        ];

        for (face, tex_opt) in faces {
            if let Some(tex) = tex_opt {
                cube.set_side_texture(context, face, &tex)?;
            }
        }

        Ok(cube)
    }

    pub fn set_side(
        context: &WgpuContext,
        target: TextureCube,
        face: CubeFace,
        texture: TextureView,
    ) -> Result<TextureCube> {
        target.set_side_texture(context, face, &texture)?;
        Ok(target)
    }

    pub fn face(context: &WgpuContext, target: TextureCube, face: CubeFace) -> Result<TextureView> {
        target.face_view_with_format(Some(context), face, target.format)
    }
    pub fn face_view(&self, face: CubeFace) -> Result<TextureView> {
        self.face_view_with_format(None, face, self.format)
    }
    pub fn face_view_with_format(
        &self,
        context: Option<&WgpuContext>,
        face: CubeFace,
        format: TextureFormat,
    ) -> Result<TextureView> {
        let texture = self
            .texture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Texture is not initialized"))?;

        let mut requested_format = format;
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = context.map(|c| c.device.push_error_scope(wgpu::ErrorFilter::Validation));

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("TextureCube Face View ({:?})", face)),
            dimension: Some(wgpu::TextureViewDimension::D2),
            format: Some(requested_format.into()),
            base_array_layer: face.index(),
            array_layer_count: Some(1),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(c), Some(scope)) = (context, error_scope) {
            let _ = c.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(err) = pollster::block_on(scope.pop()) {
                return Err(anyhow::anyhow!("WGPU TextureCube Face View Error: {}", err));
            }
        }

        Ok(TextureView {
            view: Some(Arc::new(view)),
            texture: self.texture.clone(),
            size: (self.size, self.size),
            format: requested_format,
            dimension: wgpu::TextureViewDimension::D2,
            layer: Some(face.index()),
        })
    }
    pub fn set_side_image(
        &self,
        context: &WgpuContext,
        face: CubeFace,
        image: &Image,
    ) -> Result<()> {
        let texture = self
            .texture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Texture is not initialized"))?;

        if image.width != self.size || image.height != self.size {
            return Err(anyhow::anyhow!(
                "Image size ({}x{}) must match cubemap face size ({}x{})",
                image.width,
                image.height,
                self.size,
                self.size
            ));
        }

        let pixel_size = self.format.pixel_size();
        let converted_data = image_data::convert_face_pixels(image, self.format)?;

        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: face.index(),
                },
                aspect: wgpu::TextureAspect::All,
            },
            &converted_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.size * pixel_size),
                rows_per_image: Some(self.size),
            },
            wgpu::Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }
    pub fn set_side_texture(
        &self,
        context: &WgpuContext,
        face: CubeFace,
        texture: &TextureView,
    ) -> Result<()> {
        let dst_texture = self
            .texture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Target TextureCube is not initialized"))?;

        if let Some(src_texture) = &texture.texture
            && texture.size.0 == self.size
            && texture.size.1 == self.size
            && texture.format == self.format
            && texture.layer.is_none()
        {
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
            context.queue.submit(Some(encoder.finish()));
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
        let src_view = texture
            .view
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Source TextureView view is not initialized"))?;
        let target_view_obj = self.face_view_with_format(Some(context), face, self.format)?;
        let target_view = target_view_obj
            .view
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Target TextureCube face view is not initialized"))?;

        let shader_code = r#"
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

        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("TextureCube Face Blit Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_code.into()),
            });

        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TextureCube Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout =
            context
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

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TextureCube Blit Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("TextureCube Blit Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });

        let mut target_format = self.format;
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && self.format.is_srgb() {
            target_format = self.format.linear_counterpart();
        }

        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("TextureCube Blit Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format.into(),
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
            });

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("TextureCube Blit Command Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("TextureCube Blit Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
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
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        context.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    pub fn size(&self) -> f32 {
        self.size as f32
    }

    pub fn view_2d_array(&self) -> Option<wgpu::TextureView> {
        self.texture.as_ref().map(|tex| {
            let mut requested_format = self.format;
            if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && self.format.is_srgb() {
                requested_format = self.format.linear_counterpart();
            }
            tex.create_view(&wgpu::TextureViewDescriptor {
                label: Some("TextureCube 2D Array View for Storage"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                array_layer_count: Some(6),
                format: Some(requested_format.into()),
                ..Default::default()
            })
        })
    }

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
        let storage_view = self
            .view_2d_array()
            .ok_or_else(|| anyhow::anyhow!("Failed to create 2D array view"))?;

        let wgsl_code = format!(
            r#"
            {}

            @group(0) @binding(0) var out_cube: texture_storage_2d_array<{}, write>;

            struct Uniforms {{
                time: f32,
                face: u32,
                _pad1: f32,
                _pad2: f32,
            }};
            @group(0) @binding(1) var<uniform> uniforms: Uniforms;

            fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {{
                let u = uv.x * 2.0 - 1.0;
                let v = uv.y * 2.0 - 1.0;
                var dir = vec3<f32>(0.0);
                if (face == 0u) {{ dir = vec3<f32>(1.0, -v, -u); }}
                else if (face == 1u) {{ dir = vec3<f32>(-1.0, -v, u); }}
                else if (face == 2u) {{ dir = vec3<f32>(u, 1.0, v); }}
                else if (face == 3u) {{ dir = vec3<f32>(u, -1.0, -v); }}
                else if (face == 4u) {{ dir = vec3<f32>(u, -v, 1.0); }}
                else {{ dir = vec3<f32>(-u, -v, -1.0); }}
                return normalize(dir);
            }}

            @compute @workgroup_size(8, 8, 1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
                let size = textureDimensions(out_cube).xy;
                if (id.x >= size.x || id.y >= size.y) {{
                    return;
                }}
                let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
                let dir = get_cube_direction(uv, uniforms.face);
                let color = cube(dir);
                textureStore(out_cube, id.xy, uniforms.face, color);
            }}
            "#,
            shader_src,
            self.format.to_wgsl_storage_format()
        );

        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("TextureCube Face Shader Renderer"),
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        use wgpu::util::DeviceExt;
        let uniform_bytes = [
            time.to_ne_bytes(),
            face.index().to_ne_bytes(),
            0.0f32.to_ne_bytes(),
            0.0f32.to_ne_bytes(),
        ]
        .concat();

        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("TextureCube Face Renderer Uniforms"),
                contents: &uniform_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("TextureCube Face Renderer Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: self.format.linear_counterpart().into(),
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

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TextureCube Face Renderer Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&storage_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("TextureCube Face Renderer Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("TextureCube Face Renderer Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("TextureCube Face Renderer Command Encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("TextureCube Face Renderer Compute Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_count = self.size.div_ceil(8);
            pass.dispatch_workgroups(workgroup_count, workgroup_count, 1);
        }

        context.queue.submit(Some(encoder.finish()));
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
        let storage_view = self
            .view_2d_array()
            .ok_or_else(|| anyhow::anyhow!("Failed to create 2D array view"))?;

        let wgsl_code = format!(
            r#"
            {}

            @group(0) @binding(0) var out_cube: texture_storage_2d_array<{}, write>;

            struct Uniforms {{
                time: f32,
                _pad0: f32,
                _pad1: f32,
                _pad2: f32,
            }};
            @group(0) @binding(1) var<uniform> uniforms: Uniforms;

            fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {{
                let u = uv.x * 2.0 - 1.0;
                let v = uv.y * 2.0 - 1.0;
                var dir = vec3<f32>(0.0);
                if (face == 0u) {{ dir = vec3<f32>(1.0, -v, -u); }}
                else if (face == 1u) {{ dir = vec3<f32>(-1.0, -v, u); }}
                else if (face == 2u) {{ dir = vec3<f32>(u, 1.0, v); }}
                else if (face == 3u) {{ dir = vec3<f32>(u, -1.0, -v); }}
                else if (face == 4u) {{ dir = vec3<f32>(u, -v, 1.0); }}
                else {{ dir = vec3<f32>(-u, -v, -1.0); }}
                return normalize(dir);
            }}

            @compute @workgroup_size(8, 8, 1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
                let size = textureDimensions(out_cube).xy;
                if (id.x >= size.x || id.y >= size.y || id.z >= 6u) {{
                    return;
                }}
                let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
                let dir = get_cube_direction(uv, id.z);
                let color = cube(dir);
                textureStore(out_cube, id.xy, id.z, color);
            }}
            "#,
            shader_src,
            self.format.to_wgsl_storage_format()
        );

        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("TextureCube Shader Renderer"),
                source: wgpu::ShaderSource::Wgsl(wgsl_code.into()),
            });

        use wgpu::util::DeviceExt;
        let uniforms_data = [time, 0.0, 0.0, 0.0];
        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("TextureCube Renderer Uniforms"),
                contents: bytemuck::cast_slice(&uniforms_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("TextureCube Renderer Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: self.format.linear_counterpart().into(),
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

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TextureCube Renderer Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&storage_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("TextureCube Renderer Pipeline Layout"),
                    bind_group_layouts: &[Some(&bind_group_layout)],
                    immediate_size: 0,
                });

        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("TextureCube Renderer Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        let mut encoder = context
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
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_count = self.size.div_ceil(8);
            pass.dispatch_workgroups(workgroup_count, workgroup_count, 6);
        }

        context.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

unsafe impl Send for TextureCube {}
unsafe impl Sync for TextureCube {}
