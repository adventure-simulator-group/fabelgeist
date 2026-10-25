#[cfg(not(target_arch = "wasm32"))]
use crate::globals::ValidationScope;
pub use compute_shader::*;
pub use entry_point::ComputeEntryPoint;
pub use error::*;
pub use reflection::ComputeReflection;

use crate::data::TextureFormat;
use crate::data::gpu::shader::parse_naga;
use crate::globals::WgpuContext;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod compute_shader;
mod entry_point;
mod error;
mod reflection;

type PipelineCache = HashMap<String, CachedPipeline>;

/// An immutable shader/interface definition with a shared cache of native outcomes.
#[derive(Clone, Debug)]
pub struct ComputePipeline {
    shader: ComputeShader,
    bind_group_layouts: Vec<Option<Arc<wgpu::BindGroupLayout>>>,
    reflection: Arc<ComputeReflection>,
    pipeline_cache: Arc<Mutex<PipelineCache>>,
    state: CompilationState,
}

#[derive(Clone, Copy, Debug)]
enum CompilationState {
    Prepared,
    Compiled,
}

#[derive(Clone, Debug)]
enum CachedPipeline {
    Observed(ComputePipelineResult<Arc<wgpu::ComputePipeline>>),
    DeviceHandled(Arc<wgpu::ComputePipeline>),
}

impl CachedPipeline {
    fn outcome(&self) -> ComputePipelineResult<Arc<wgpu::ComputePipeline>> {
        match self {
            Self::Observed(outcome) => outcome.clone(),
            Self::DeviceHandled(pipeline) => Ok(pipeline.clone()),
        }
    }
}

impl ComputePipeline {
    pub fn new(
        context: &WgpuContext,
        shader: ComputeShader,
        entry: ComputeEntryPoint<'_>,
    ) -> ComputePipelineResult<Self> {
        let mut pipeline = Self::prepare(context, shader, entry)?;
        pipeline.compile(context)?;
        Ok(pipeline)
    }

    /// Reflect bindings without compiling a GPU pipeline. A caller choosing a
    /// different uniform layout can compile only the dispatch layout it uses.
    pub fn prepare(
        context: &WgpuContext,
        shader: ComputeShader,
        entry: ComputeEntryPoint<'_>,
    ) -> ComputePipelineResult<Self> {
        if shader.device() != context.device.as_ref() {
            return Err(ComputePipelineError::WrongDevice);
        }
        let interface = reflection::ReflectedInterface::new(context, shader.code(), entry)?;
        Ok(Self {
            shader,
            bind_group_layouts: interface.layouts,
            reflection: interface.reflection,
            pipeline_cache: Arc::new(Mutex::new(HashMap::new())),
            state: CompilationState::Prepared,
        })
    }

    pub fn shader(&self) -> &ComputeShader {
        &self.shader
    }
    pub fn reflection(&self) -> &ComputeReflection {
        &self.reflection
    }
    pub fn bind_group_layouts(&self) -> &[Option<Arc<wgpu::BindGroupLayout>>] {
        &self.bind_group_layouts
    }
    pub fn device(&self) -> &wgpu::Device {
        self.shader.device()
    }

    pub fn ensure_device(&self, context: &WgpuContext) -> ComputePipelineResult<()> {
        if self.device() != context.device.as_ref() {
            return Err(ComputePipelineError::WrongDevice);
        }
        Ok(())
    }

    pub fn compiled(
        &self,
        context: &WgpuContext,
    ) -> ComputePipelineResult<Arc<wgpu::ComputePipeline>> {
        self.ensure_device(context)?;
        if matches!(self.state, CompilationState::Prepared) {
            return Err(ComputePipelineError::NotCompiled);
        }
        self.get_or_create_pipeline(context, &self.reflection.entry_point)
    }

    /// Native validation follows the supplied context's existing blocking policy.
    /// Completed native validation outcomes are cached, including rejection.
    /// Failed progress remains uncached and can be retried.
    pub fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        entry_point: &str,
    ) -> ComputePipelineResult<Arc<wgpu::ComputePipeline>> {
        self.ensure_device(context)?;
        let mut cache = self
            .pipeline_cache
            .lock()
            .map_err(|_| ComputePipelineError::CachePoisoned)?;
        if let Some(cached) = cache.get(entry_point) {
            // A native context requesting readback must not inherit an earlier
            // borrowed context's unchecked creation merely because devices match.
            if matches!(cached, CachedPipeline::Observed(_)) || !observes_validation(context) {
                return cached.outcome();
            }
        }
        let cached = match self.create_native(context, entry_point) {
            Ok(cached) => cached,
            Err(cause) => {
                cache.remove(entry_point);
                return Err(cause);
            }
        };
        let outcome = cached.outcome();
        cache.insert(entry_point.to_owned(), cached);
        outcome
    }

    /// Compile this definition's static uniform layout on first use. Clones
    /// share completed entry-point outcomes, including native rejection.
    /// Failed progress does not complete compilation and remains retryable.
    pub fn compile(&mut self, context: &WgpuContext) -> ComputePipelineResult<()> {
        self.ensure_device(context)?;
        self.get_or_create_pipeline(context, &self.reflection.entry_point)?;
        self.state = CompilationState::Compiled;
        Ok(())
    }

    pub fn validate_interface(&self) -> ComputePipelineResult<()> {
        let module = parse_naga(self.shader.code(), wgpu::naga::ShaderStage::Compute)?;
        ComputeEntryPoint::Named(&self.reflection.entry_point).resolve(&module)?;
        Ok(())
    }

    fn create_native(
        &self,
        context: &WgpuContext,
        entry_point: &str,
    ) -> ComputePipelineResult<CachedPipeline> {
        #[cfg(not(target_arch = "wasm32"))]
        let scope = ValidationScope::new(context);
        let layouts: Vec<_> = self
            .bind_group_layouts
            .iter()
            .map(|layout| layout.as_deref())
            .collect();
        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ComputePipeline Layout"),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            });
        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ComputePipeline"),
                layout: Some(&layout),
                module: self.shader.module(),
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: None,
            });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            if let Err(cause) = scope.finish() {
                let cause = ComputePipelineError::from(cause);
                if matches!(cause, ComputePipelineError::Progress(_)) {
                    return Err(cause);
                }
                return Ok(CachedPipeline::Observed(Err(cause)));
            }
        }
        let pipeline = Arc::new(pipeline);
        if observes_validation(context) {
            Ok(CachedPipeline::Observed(Ok(pipeline)))
        } else {
            Ok(CachedPipeline::DeviceHandled(pipeline))
        }
    }
}

fn observes_validation(context: &WgpuContext) -> bool {
    cfg!(not(target_arch = "wasm32")) && context.blocking_validation
}
