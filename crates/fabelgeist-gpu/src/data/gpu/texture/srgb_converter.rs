use crate::data::gpu::texture::{Texture2d, TextureViewError};
use crate::data::{ComputePassError, ComputePipelineError, ComputeShader};
use crate::globals::WgpuContext;
use anyhow::{Context, Result};
use std::sync::Arc;

// Native dispatch dimensions match the fixed conversion shader below.
const CONVERSION_WORKGROUP_SIDE: u32 = 16;
const CONVERSION_SHADER: &str = r#"
@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var dst_tex: texture_storage_2d<rgba8unorm, write>;

fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let a = 12.92 * linear;
    let b = 1.055 * pow(linear, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(b, a, linear <= vec3<f32>(0.0031308));
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(src_tex);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    let pos = id.xy;
    let color = textureLoad(src_tex, pos, 0);
    let srgb_rgb = linear_to_srgb(color.rgb);
    textureStore(dst_tex, pos, vec4<f32>(srgb_rgb, color.a));
}
"#;

#[derive(Clone)]
pub struct TextureSrgbConverter {
    pipeline: Arc<wgpu::ComputePipeline>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
    device: Arc<wgpu::Device>,
}

impl TextureSrgbConverter {
    pub fn new(context: &WgpuContext) -> Result<Self> {
        let device = &context.device;
        let shader = ComputeShader::new(context, CONVERSION_SHADER.to_owned())?;
        #[cfg(not(target_arch = "wasm32"))]
        let scope = context
            .blocking_validation
            .then(|| device.push_error_scope(wgpu::ErrorFilter::Validation));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sRGB Conversion Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sRGB Conversion Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("sRGB Conversion Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader.module(),
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ComputePipelineError::Native(Arc::new(cause)).into());
            }
        }
        Ok(Self {
            pipeline: Arc::new(pipeline),
            bind_group_layout: Arc::new(bind_group_layout),
            device: context.device.clone(),
        })
    }

    pub fn convert(&self, context: &WgpuContext, src: &Texture2d, dst: &Texture2d) -> Result<()> {
        let device = &context.device;
        // Preserve source-then-destination cache absence before device admission.
        src.cached_view()
            .ok_or(TextureViewError::MissingView)
            .context("Source texture view missing")?;
        dst.cached_view()
            .ok_or(TextureViewError::MissingView)
            .context("Destination texture view missing")?;
        src.ensure_device(context)?;
        dst.ensure_device(context)?;
        if self.device.as_ref() != device.as_ref() {
            return Err(ComputePipelineError::WrongDevice.into());
        }
        let src_view = src.select_view(context, src.format())?;
        let dst_view = dst.select_view(context, dst.format())?;
        #[cfg(not(target_arch = "wasm32"))]
        let scope = context
            .blocking_validation
            .then(|| device.push_error_scope(wgpu::ErrorFilter::Validation));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sRGB Conversion Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst_view.view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sRGB Conversion Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("sRGB Conversion Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);

            // Cover the fixed conversion shader's invocation dimensions.
            let workgroups_x = src.extent().width.div_ceil(CONVERSION_WORKGROUP_SIDE);
            let workgroups_y = src.extent().height.div_ceil(CONVERSION_WORKGROUP_SIDE);
            cpass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        let commands = encoder.finish();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ComputePassError::Native(Arc::new(cause)).into());
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let scope = context
            .blocking_validation
            .then(|| device.push_error_scope(wgpu::ErrorFilter::Validation));
        context.queue.submit(Some(commands));
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ComputePassError::Native(Arc::new(cause)).into());
            }
        }
        Ok(())
    }
}
