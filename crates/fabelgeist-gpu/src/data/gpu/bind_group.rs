pub(crate) use assembly::create_pipeline_bind_groups;

use crate::data::{PassParameters, ReflectionData};
use crate::globals::WgpuContext;
use anyhow::{Context, Result};

mod assembly;
mod uniforms;

/// A native group retains its reflected index, including gaps between groups.
#[derive(Debug)]
pub struct ComputeBindGroup {
    index: u32,
    group: wgpu::BindGroup,
}

impl ComputeBindGroup {
    pub fn bind(&self, pass: &mut wgpu::ComputePass<'_>) {
        pass.set_bind_group(self.index, &self.group, &[]);
    }
}

/// Assemble reflected static bindings, preserving logical buffer prefixes and
/// the selected texture view's effective format, dimension and creating device.
pub fn create_bind_groups(
    context: &WgpuContext,
    bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
    reflection: &ReflectionData,
    parameters: &PassParameters,
    pass_name: &str,
) -> Result<Vec<ComputeBindGroup>> {
    reflection
        .bind_groups
        .iter()
        .map(|reflection| {
            let layout = bind_group_layouts
                .get(reflection.index as usize)
                .copied()
                .flatten()
                .ok_or(crate::data::ComputePassError::MissingLayout {
                    index: reflection.index,
                })?;
            let group = assembly::create_group(context, layout, reflection, parameters)
                .with_context(|| format!("{pass_name}: bind group {}", reflection.index))?;
            Ok(ComputeBindGroup {
                index: reflection.index,
                group,
            })
        })
        .collect()
}
