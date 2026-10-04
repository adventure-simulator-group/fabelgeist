mod entry_key;
mod error;
mod reflection;
pub use entry_key::PipelineEntryKey;
pub use error::ComputePipelineError;
mod compute_shader;
pub use compute_shader::*;

use crate::data::gpu::shader::{ReflectionData, ShaderEntryPoint};
use crate::globals::WgpuContext;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ComputePipeline {
    pub shader: ComputeShader,
    pub bind_group_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    pub reflection: Option<Arc<ReflectionData>>,
    pub pipeline_cache: Arc<Mutex<HashMap<PipelineEntryKey, Arc<wgpu::ComputePipeline>>>>,
    pub pipeline: Option<Arc<wgpu::ComputePipeline>>,
    pub validation_error: Arc<Mutex<Option<Arc<wgpu::Error>>>>,
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
    /// Build the complete pipeline from admitted module source, retaining the
    /// shader-preparation stage if it fails before pipeline construction.
    pub fn from_source(
        context: &WgpuContext,
        source: crate::data::gpu::shader::ShaderSource,
    ) -> Result<Self, ComputePipelineError> {
        let shader = ComputeShader::new(context, source).map_err(ComputePipelineError::Shader)?;
        Self::new(context, shader)
    }

    pub fn get_or_create_pipeline(
        &self,
        device: &wgpu::Device,
        entry_point: &ShaderEntryPoint,
    ) -> Result<Arc<wgpu::ComputePipeline>, ComputePipelineError> {
        self.get_or_create_pipeline_validated(
            device,
            entry_point,
            crate::globals::ValidationReadback::Blocking,
        )
    }

    /// The same, with control over whether the validation error scope is read
    /// back. Draining the queue to read it is only safe on a device nothing
    /// else is submitting to -- see `WgpuContext::validation_readback`.
    pub fn get_or_create_pipeline_validated(
        &self,
        device: &wgpu::Device,
        entry_point: &ShaderEntryPoint,
        validation_readback: crate::globals::ValidationReadback,
    ) -> Result<Arc<wgpu::ComputePipeline>, ComputePipelineError> {
        let cache_key = PipelineEntryKey::from(entry_point);

        let mut cache = self.pipeline_cache.lock().map_err(
            |_: std::sync::PoisonError<
                std::sync::MutexGuard<'_, HashMap<PipelineEntryKey, Arc<wgpu::ComputePipeline>>>,
            >|
             -> ComputePipelineError { ComputePipelineError::CachePoisoned },
        )?;
        if let Some(p) = cache.get(&cache_key) {
            return Ok(p.clone());
        }

        // Reset validation error
        if let Ok(mut guard) = self.validation_error.lock() {
            *guard = None;
        }

        // Create Pipeline
        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = validation_readback.scope(device);

        let module = self
            .shader
            .module
            .as_ref()
            .ok_or(ComputePipelineError::MissingModule)?;

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
            entry_point: Some(<&str>::from(entry_point)),
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
                *guard = Some(Arc::new(e));
            }
        }

        let p_arc = Arc::new(pipeline);
        cache.insert(cache_key, p_arc.clone());
        Ok(p_arc)
    }

    pub fn validate_interface(&self) -> Result<(), ComputePipelineError> {
        let naga_res = self
            .shader
            .code
            .parse(wgpu::naga::ShaderStage::Compute)
            .map_err(ComputePipelineError::Interface)?;

        if !naga_res
            .entry_points
            .iter()
            .any(|ep| ep.stage == wgpu::naga::ShaderStage::Compute)
        {
            return Err(ComputePipelineError::MissingComputeEntryPoint);
        }

        Ok(())
    }
}

impl ComputePipeline {
    pub fn new(
        context: &WgpuContext,
        shader: ComputeShader,
    ) -> Result<ComputePipeline, ComputePipelineError> {
        let reflected = reflection::ReflectedComputeModule::new(&shader.code)?;
        let bind_group_layouts = reflected.create_layouts(&context.device);
        let mut pipeline = Self {
            shader: shader.clone(),
            bind_group_layouts,
            reflection: Some(Arc::new(reflected.into_data())),
            ..Default::default()
        };

        // Ensure shader is up to date
        pipeline.shader = shader.clone();

        // Bake WGPU Resource
        // 1. Synchronous Naga Validation
        pipeline.validate_interface()?;

        // 2. WGPU Bake
        let reflection = pipeline.reflection.as_ref().unwrap();
        let entry = &reflection.compute_entry_point;

        if let Ok(p_wgpu) = pipeline.get_or_create_pipeline_validated(
            &context.device,
            entry,
            context.validation_readback,
        ) {
            pipeline.pipeline = Some(p_wgpu);
        }

        if let Ok(guard) = pipeline.validation_error.lock()
            && let Some(err) = guard.as_ref()
        {
            return Err(ComputePipelineError::Device(err.clone()));
        }

        Ok(pipeline)
    }
}
