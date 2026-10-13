use crate::data::gpu::shader::{ShaderLanguage, ShaderSource, parse_naga};
use crate::globals::WgpuContext;
use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub struct ComputeShader {
    pub code: ShaderSource<'static>,
    pub module: Option<Arc<wgpu::ShaderModule>>,
    pub error: Arc<Mutex<Option<String>>>,
}
impl ComputeShader {
    pub fn new(context: &WgpuContext, code: ShaderSource<'_>) -> Result<ComputeShader> {
        let code = code.into_owned();
        // 1. Naga Parse & Deep Validation
        let language = code.language();
        let naga_res = parse_naga(&code, wgpu::naga::ShaderStage::Compute)
            .map_err(|e| anyhow::anyhow!("Compute Shader Parse Error: {}", e))?;

        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        );

        let info = if let Ok(info) = validator.validate(&naga_res) {
            info
        } else {
            let e = validator.validate(&naga_res).unwrap_err();
            let message = e.emit_to_string(code.as_str());
            return Err(anyhow::anyhow!(
                "Compute Shader Validation Error: {}",
                message
            ));
        };

        // 2. Convert to WGSL for WGPU compatibility (if it was GLSL)
        let wgsl_code = match language {
            ShaderLanguage::Glsl => match wgpu::naga::back::wgsl::write_string(
                &naga_res,
                &info,
                wgpu::naga::back::wgsl::WriterFlags::empty(),
            ) {
                Ok(s) => s,
                Err(e) => return Err(anyhow!("Failed to convert GLSL to WGSL: {}", e)),
            },
            ShaderLanguage::Wgsl => code.as_str().to_owned(),
        };

        // 3. WGPU Validation & Creation
        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = context.blocking_validation.then(|| {
            context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });

        let sm = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ComputeShader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&wgsl_code)),
            });
        let module = Some(Arc::new(sm));

        let error = Arc::new(Mutex::new(None));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(e) = pollster::block_on(error_scope.pop()) {
                return Err(anyhow!("Compute Shader Validation Error: {}", e));
            }
        }

        let definition = ComputeShader {
            code,
            module,
            error,
        };

        Ok(definition)
    }
}
impl PartialEq for ComputeShader {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}
