use crate::data::gpu::shader::{ShaderLanguage, ShaderParseError, ShaderSource};
use crate::globals::WgpuContext;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ComputeShader {
    pub code: ShaderSource,
    pub module: Option<Arc<wgpu::ShaderModule>>,
}

/// CPU preparation preserves the original source separately from emitted WGSL.
#[derive(Debug)]
pub struct PreparedComputeShader {
    source: ShaderSource,
    gpu_source: ShaderSource,
}
impl PreparedComputeShader {
    pub fn new(source: ShaderSource) -> Result<Self, ComputeShaderError> {
        let module = source
            .parse(wgpu::naga::ShaderStage::Compute)
            .map_err(ComputeShaderError::Parse)?;
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        );
        let info = validator.validate(&module).map_err(
            |cause: wgpu::naga::WithSpan<wgpu::naga::valid::ValidationError>| -> ComputeShaderError {
                ComputeShaderError::Validation {
                    source: source.clone(),
                    cause: Box::new(cause),
                }
            },
        )?;
        let gpu_source = match source.language() {
            ShaderLanguage::Glsl => ShaderSource::from(
                wgpu::naga::back::wgsl::write_string(
                    &module,
                    &info,
                    wgpu::naga::back::wgsl::WriterFlags::empty(),
                )
                .map_err(ComputeShaderError::Conversion)?,
            ),
            ShaderLanguage::Wgsl => source.clone(),
        };
        Ok(Self { source, gpu_source })
    }
    pub fn source(&self) -> &ShaderSource {
        &self.source
    }
    pub fn gpu_source(&self) -> &ShaderSource {
        &self.gpu_source
    }
}

impl ComputeShader {
    pub fn new(context: &WgpuContext, code: ShaderSource) -> Result<Self, ComputeShaderError> {
        let prepared = PreparedComputeShader::new(code)?;
        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = context.validation_readback.scope(&context.device);
        let module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ComputeShader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(<&str>::from(
                    prepared.gpu_source(),
                ))),
            });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(error_scope.pop()) {
                return Err(ComputeShaderError::Device(cause));
            }
        }
        Ok(Self {
            code: prepared.source,
            module: Some(Arc::new(module)),
        })
    }
}
impl PartialEq for ComputeShader {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

#[derive(Debug)]
pub enum ComputeShaderError {
    Parse(ShaderParseError),
    Validation {
        source: ShaderSource,
        cause: Box<wgpu::naga::WithSpan<wgpu::naga::valid::ValidationError>>,
    },
    Conversion(wgpu::naga::back::wgsl::Error),
    Device(wgpu::Error),
}
impl std::fmt::Display for ComputeShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(cause) => write!(f, "Compute Shader Parse Error: {cause}"),
            Self::Validation { source, cause } => write!(
                f,
                "Compute Shader Validation Error: {}",
                cause.emit_to_string(<&str>::from(source))
            ),
            Self::Conversion(cause) => write!(f, "Failed to convert GLSL to WGSL: {cause}"),
            Self::Device(cause) => write!(f, "Compute Shader Validation Error: {cause}"),
        }
    }
}
impl std::error::Error for ComputeShaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(cause) => Some(cause),
            Self::Validation { cause, .. } => Some(cause.as_ref()),
            Self::Conversion(cause) => Some(cause),
            Self::Device(cause) => Some(cause),
        }
    }
}
