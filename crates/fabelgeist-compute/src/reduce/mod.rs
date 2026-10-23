use crate::prelude::*;
use fabelgeist_gpu::data::gpu::ResourceDescriptor;

pub use execution::ReduceScratchpad;
pub use max::Max;
pub use min::Min;

use fabelgeist_gpu::prelude::BufferElementStride;
use layout::REDUCTION_WORKGROUP_WIDTH;

mod execution;
mod layout;
pub mod max;
pub mod min;

#[derive(Clone, Debug)]
pub struct ReduceDefinition {
    pub code: String,
    cache: ComputeSourceCache<ResourceDescriptor, ReductionPipeline>,
}

/// Records a reduction into one returned native record.
pub struct Reduce;

#[derive(Clone, Debug)]
struct ReductionPipeline {
    pipeline: ComputePipeline,
    stride: BufferElementStride,
}

impl Default for ReduceDefinition {
    fn default() -> Self {
        Self {
            code: String::new(),
            cache: ComputeSourceCache::default(),
        }
    }
}

impl PartialEq for ReduceDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl ReduceDefinition {
    pub fn new(_context: &WgpuContext, code: String) -> Result<Self> {
        let _ = fabelgeist_gpu::data::gpu::signature::parse_binary_op_signature(&code, "reduce")?;
        Ok(Self {
            code,
            cache: ComputeSourceCache::default(),
        })
    }

    fn get_or_create_pipeline(
        &self,
        context: &WgpuContext,
        input_res: ResourceDescriptor,
    ) -> Result<ReductionPipeline> {
        self.cache.get_or_try_insert(context, &self.code, input_res.clone(), || {
            let ty =
                fabelgeist_gpu::data::gpu::signature::parse_binary_op_signature(&self.code, "reduce")?;
            let ty_str = ty.as_str();

            let input_binding = input_res.to_wgsl_input_binding(0, 0, "input");
            let output_binding = format!(
                "@group(0) @binding(1) var<storage, read_write> output: array<{}>;\n",
                ty_str
            );

            let prologue = match input_res {
                ResourceDescriptor::Buffer(_) => "".to_string(),
                ResourceDescriptor::Texture2d(_) => {
                    "let _dim = textureDimensions(input);\n".to_string()
                }
                ResourceDescriptor::Texture3d(_) => {
                    "let _dim = textureDimensions(input);\n".to_string()
                }
            };

            let input_len_calc = match input_res {
                ResourceDescriptor::Buffer(_) => "arrayLength(&input)".to_string(),
                ResourceDescriptor::Texture2d(_) => "(_dim.x * _dim.y)".to_string(),
                ResourceDescriptor::Texture3d(_) => "(_dim.x * _dim.y * _dim.z)".to_string(),
            };

            // Helper to fetch value from any resource type given a flat index
            let fetch_logic = match input_res {
                ResourceDescriptor::Buffer(_) => "input[index]".to_string(),
                ResourceDescriptor::Texture2d(_) => {
                    let swizzle = match ty.component_count() {
                        1 => ".x",
                        2 => ".xy",
                        _ => "",
                    };
                    format!(
                        "textureLoad(input, vec2<u32>(index % _dim.x, index / _dim.x), 0){}",
                        swizzle
                    )
                }
                ResourceDescriptor::Texture3d(_) => {
                    let swizzle = match ty.component_count() {
                        1 => ".x",
                        2 => ".xy",
                        _ => "",
                    };
                    format!(
                        "textureLoad(input, vec3<u32>(index % _dim.x, (index / _dim.x) % _dim.y, index / (_dim.x * _dim.y)), 0){}",
                        swizzle
                    )
                }
            };

            let full_code = format!(
                r#"{}
{}

var<workgroup> shared_data: array<{}, {width}>;

{}

@compute @workgroup_size({width})
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>
) {{
    {}
    let input_len = {};
    let index = global_id.x;

    if (index < input_len) {{
        shared_data[local_id.x] = {};
    }}

    workgroupBarrier();

    for (var s = {half_width}u; s > 0u; s >>= 1u) {{
        if (local_id.x < s) {{
            let idx_a = local_id.x;
            let idx_b = local_id.x + s;

            let global_idx_b = group_id.x * {width}u + idx_b;

            if (global_idx_b < input_len) {{
                shared_data[idx_a] = reduce(shared_data[idx_a], shared_data[idx_b]);
            }}
        }}
        workgroupBarrier();
    }}

    if (local_id.x == 0u) {{
        output[group_id.x] = shared_data[0];
    }}
}}
"#,
                input_binding,
                output_binding,
                ty_str,
                self.code,
                prologue,
                input_len_calc,
                fetch_logic,
                width = REDUCTION_WORKGROUP_WIDTH,
                half_width = REDUCTION_WORKGROUP_WIDTH / 2,
            );

            let module = fabelgeist_gpu::data::gpu::shader::parse_naga(
                &full_code,
                wgpu::naga::ShaderStage::Compute,
            )?;

            let stride = module
                .global_variables
                .iter()
                .find(|(_, variable)| variable.name.as_deref() == Some("output"))
                .and_then(|(_, variable)| match &module.types[variable.ty].inner {
                    wgpu::naga::TypeInner::Array { stride, .. } => Some(*stride),
                    _ => None,
                })
                .ok_or_else(|| anyhow!("Reduction output has no native array layout"))?;
            let stride = BufferElementStride::try_from(stride)?;

            let shader = ComputeShader::new(context, full_code)?;
            let pipeline = ComputePipeline::new(context, shader)?;
            Ok(ReductionPipeline { pipeline, stride })
        })
    }
}

#[cfg(test)]
mod tests;
