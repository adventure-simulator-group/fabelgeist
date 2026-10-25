use crate::data::gpu::shader::{
    ShaderError, ShaderLanguage, ShaderResult, detect_from_code, parse_naga,
};
use crate::globals::WgpuContext;
use std::sync::Arc;

/// Source, module and creating device are admitted together and remain immutable.
#[derive(Clone, Debug)]
pub struct ComputeShader {
    code: String,
    module: Arc<wgpu::ShaderModule>,
    device: Arc<wgpu::Device>,
}

impl ComputeShader {
    pub fn new(context: &WgpuContext, code: String) -> ShaderResult<Self> {
        let module = parse_naga(&code, wgpu::naga::ShaderStage::Compute)?;
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        );
        let info = validator
            .validate(&module)
            .map_err(|cause| ShaderError::Validation {
                diagnostic: cause.emit_to_string(&code),
                cause: Arc::new(cause),
            })?;
        let wgsl_code = match detect_from_code(&code) {
            ShaderLanguage::Glsl => wgpu::naga::back::wgsl::write_string(
                &module,
                &info,
                wgpu::naga::back::wgsl::WriterFlags::empty(),
            )
            .map_err(|cause| ShaderError::Translation(Arc::new(cause)))?,
            ShaderLanguage::Wgsl => code.clone(),
        };
        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = context.blocking_validation.then(|| {
            context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });
        let module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ComputeShader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&wgsl_code)),
            });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = error_scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ShaderError::Native(Arc::new(cause)));
            }
        }
        Ok(Self {
            code,
            module: Arc::new(module),
            device: context.device.clone(),
        })
    }

    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}

impl PartialEq for ComputeShader {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}
