use super::{EmptyReductionResult, Reduce, ReduceDefinition};
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::signature::ResourceBaseType;

pub struct Min;

impl Min {
    pub fn execute(
        context: &WgpuContext,
        input: &fabelgeist_gpu::data::gpu::resource::GpuResource,
        custom_code: Option<&str>,
        scratchpad: &mut super::ReduceScratchpad,
    ) -> Result<fabelgeist_gpu::data::gpu::Buffer> {
        let code = if let Some(custom) = custom_code {
            let ty = fabelgeist_gpu::data::gpu::signature::parse_compare_signature(custom, "min")?;
            let ty_str = ty.as_str();
            format!(
                r#"
{}
fn reduce(a: {}, b: {}) -> {} {{
    if (min_cmp(a, b)) {{ return a; }} else {{ return b; }}
}}
"#,
                custom.replace("fn min(", "fn min_cmp("),
                ty_str,
                ty_str,
                ty_str
            )
        } else {
            let ty_str = input.base_type().as_str();
            format!(
                "fn reduce(a: {}, b: {}) -> {} {{ return min(a, b); }}",
                ty_str, ty_str, ty_str
            )
        };

        let definition = ReduceDefinition::new(context, code)?;
        Reduce::execute(context, &definition, input, scratchpad)
    }

    /// Read the first component of the reduced native record as a number.
    ///
    /// Returns [`EmptyReductionResult`] when no component exists.
    pub async fn execute_to_number(
        context: &WgpuContext,
        input: &fabelgeist_gpu::data::gpu::resource::GpuResource,
        scratchpad: &mut super::ReduceScratchpad,
    ) -> Result<f64> {
        let buffer = Self::execute(context, input, None, scratchpad)?;
        let base_type = input.base_type();

        match base_type.base_type() {
            ResourceBaseType::F32 => {
                let data: Vec<f32> = buffer.read(context).await?;
                Ok(f64::from(*data.first().ok_or(EmptyReductionResult)?))
            }
            ResourceBaseType::U32 => {
                let data: Vec<u32> = buffer.read(context).await?;
                Ok(f64::from(*data.first().ok_or(EmptyReductionResult)?))
            }
            ResourceBaseType::I32 => {
                let data: Vec<i32> = buffer.read(context).await?;
                Ok(f64::from(*data.first().ok_or(EmptyReductionResult)?))
            }
            _ => Err(anyhow!(
                "Reduction to a single Number is only supported for scalar types (f32, u32, i32) or vectors of them. Got: {}",
                base_type.as_str()
            )),
        }
    }
}
