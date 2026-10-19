mod compute_shader;
mod reflection;
pub use compute_shader::*;

use crate::data::TextureFormat;
use crate::data::gpu::shader::{ReflectionData, parse_naga};
use crate::globals::WgpuContext;
use anyhow::{Result, anyhow};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ComputePipeline {
    pub shader: ComputeShader,
    pub bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    pub reflection: Option<Arc<ReflectionData>>,
    pub pipeline_cache: Arc<Mutex<HashMap<u64, Arc<wgpu::ComputePipeline>>>>,
    pub pipeline: Option<Arc<wgpu::ComputePipeline>>,
    pub validation_error: Arc<Mutex<Option<String>>>,
}

impl Default for ComputePipeline {
    fn default() -> Self {
        Self {
            shader: ComputeShader::default(),
            bind_group_layouts: Vec::new(),
            reflection: None,
            pipeline_cache: Arc::new(Mutex::new(HashMap::new())),
            pipeline: None,
            validation_error: Arc::new(Mutex::new(None)),
        }
    }
}

impl ComputePipeline {
    pub fn get_or_create_pipeline(
        &self,
        device: &wgpu::Device,
        entry_point: &str,
    ) -> anyhow::Result<Arc<wgpu::ComputePipeline>> {
        self.get_or_create_pipeline_validated(device, entry_point, true)
    }

    /// The same, with control over whether the validation error scope is read
    /// back. Draining the queue to read it is only safe on a device nothing
    /// else is submitting to -- see `WgpuContext::blocking_validation`.
    pub fn get_or_create_pipeline_validated(
        &self,
        device: &wgpu::Device,
        entry_point: &str,
        // The web never blocks to read the scope back.
        #[cfg_attr(target_arch = "wasm32", allow(unused_variables))] blocking_validation: bool,
    ) -> anyhow::Result<Arc<wgpu::ComputePipeline>> {
        #[cfg(target_arch = "wasm32")]
        let _ = blocking_validation;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Cache key based on entry point (shader code is part of shader struct but not strictly hashed here, assuming one pipeline per shadernode usually)
        // Ideally we should hash the shader code or module ID too if it changes.
        // For now lets hash the entry point.
        let mut s = DefaultHasher::new();
        entry_point.hash(&mut s);
        let cache_key = s.finish();

        let mut cache = self
            .pipeline_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock pipeline cache"))?;
        if let Some(p) = cache.get(&cache_key) {
            return Ok(p.clone());
        }

        // Reset validation error
        if let Ok(mut guard) = self.validation_error.lock() {
            *guard = None;
        }

        // Create Pipeline
        #[cfg(not(target_arch = "wasm32"))]
        let error_scope =
            blocking_validation.then(|| device.push_error_scope(wgpu::ErrorFilter::Validation));

        let module = self
            .shader
            .module
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("ComputePipeline: Shader Module missing"))?;

        let layouts: Vec<_> = self.bind_group_layouts.iter().map(|l| l.as_ref()).collect();
        let sparse_layouts: Vec<Option<&wgpu::BindGroupLayout>> =
            layouts.iter().map(|&l| Some(l)).collect();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ComputePipeline Layout"),
            bind_group_layouts: &sparse_layouts,
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ComputePipeline"),
            layout: Some(&pipeline_layout),
            module,
            entry_point: Some(entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        // POP ERROR SCOPE
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(e) = pollster::block_on(error_scope.pop())
                && let Ok(mut guard) = self.validation_error.lock()
            {
                *guard = Some(e.to_string());
            }
        }

        let p_arc = Arc::new(pipeline);
        cache.insert(cache_key, p_arc.clone());
        Ok(p_arc)
    }

    pub fn validate_interface(&self) -> anyhow::Result<()> {
        let naga_res = parse_naga(&self.shader.code, wgpu::naga::ShaderStage::Compute)
            .map_err(|e| anyhow::anyhow!("Compute Shader Parse Error: {}", e))?;

        if !naga_res
            .entry_points
            .iter()
            .any(|ep| ep.stage == wgpu::naga::ShaderStage::Compute)
        {
            return Err(anyhow::anyhow!("Compute Shader missing entry point"));
        }

        Ok(())
    }
}

impl ComputePipeline {
    pub fn new(context: &WgpuContext, shader: ComputeShader) -> Result<Self> {
        let mut pipeline = Self::prepare(context, shader)?;
        pipeline.compile(context)?;
        Ok(pipeline)
    }

    /// Reflect bindings without compiling a GPU pipeline. A caller choosing a
    /// different uniform layout can share the shader and its interface, then
    /// compile only the dispatch layout it actually uses.
    pub fn prepare(context: &WgpuContext, shader: ComputeShader) -> Result<Self> {
        let interface = reflection::ReflectedInterface::new(context, &shader.code)?;
        Ok(Self {
            shader,
            bind_group_layouts: interface.layouts,
            reflection: Some(interface.reflection),
            ..Default::default()
        })
    }

    /// Compile this definition's static uniform layout on first use. Clones
    /// share the entry-point cache, including definitions used by raw kernels.
    pub fn compile(&mut self, context: &WgpuContext) -> Result<()> {
        if self.pipeline.is_some() {
            return Ok(());
        }
        let reflection = self
            .reflection
            .as_ref()
            .ok_or_else(|| anyhow!("ComputePipeline: Reflection missing"))?;
        let pipeline = self.get_or_create_pipeline_validated(
            &context.device,
            &reflection.vertex_entry_point,
            context.blocking_validation,
        )?;
        let validation = self
            .validation_error
            .lock()
            .map_err(|_| anyhow!("Failed to lock pipeline validation state"))?;
        if let Some(error) = validation.as_ref() {
            return Err(anyhow!("ComputePipeline Creation Error: {error}"));
        }
        self.pipeline = Some(pipeline);
        Ok(())
    }
}
