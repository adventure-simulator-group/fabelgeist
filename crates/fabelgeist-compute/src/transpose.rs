use crate::prelude::*;

use fabelgeist_gpu::data::gpu::resource::GpuResource;

#[derive(Clone, Debug, Default)]
pub struct TransposeDefinition {
    cache: ComputeSourceCache<(), ComputePipeline>,
}

impl PartialEq for TransposeDefinition {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl TransposeDefinition {
    pub fn new(_context: &WgpuContext, _code: String) -> Result<Self> {
        Ok(Self {
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn get_or_create_pipeline(&self, context: &WgpuContext) -> Result<ComputePipeline> {
        let shader_code = "
            @group(0) @binding(0) var<storage, read> input: array<f32>;
            @group(0) @binding(1) var<storage, read_write> output: array<f32>;
            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                output[global_id.x] = input[global_id.x];
            }
        ";

        self.cache.get_or_try_insert(context, shader_code, (), || {
            let shader = ComputeShader::new(context, shader_code.to_string())?;
            let pipeline = fabelgeist_gpu::data::gpu::build_compute_pipeline(context, &shader, "main")?;
            Ok(pipeline)
        })
    }
}

pub struct Transpose;

impl Transpose {
    pub fn execute(
        context: &WgpuContext,
        definition: &TransposeDefinition,
        input: &GpuResource,
        output: &GpuResource,
    ) -> Result<()> {
        let pipeline = definition.get_or_create_pipeline(context)?;

        let mut parameters = fabelgeist_gpu::data::gpu::parameters::PassParameters::new();
        match input {
            GpuResource::Buffer(b) => parameters.insert("input", b.clone()),
            _ => return Err(anyhow::anyhow!("Transpose input must be a buffer")),
        }
        match output {
            GpuResource::Buffer(b) => parameters.insert("output", b.clone()),
            _ => return Err(anyhow::anyhow!("Transpose output must be a buffer")),
        }

        let wg_x = match output {
            GpuResource::Buffer(b) => ((u64::from(b.size) / 4) as u32).div_ceil(64),
            _ => 1,
        };

        fabelgeist_gpu::data::gpu::ComputePass::dispatch(
            context, pipeline, parameters, wg_x, 1, 1,
        )?;

        Ok(())
    }
}
