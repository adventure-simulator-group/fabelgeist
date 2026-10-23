pub use assembly::create_bind_groups;

mod assembly;
mod uniform_buffer;
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
