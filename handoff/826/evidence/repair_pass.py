from pathlib import Path
import subprocess,re
base=Path('crates/fabelgeist-gpu/src/data/gpu/compute/pass')
for name in ['grid.rs','shape.rs']:
 s=subprocess.check_output(['git','show','71f9cab8a5f97760e2e61ca4c74876b715f8aebc:crates/fabelgeist-gpu/src/data/gpu/compute/pass/'+name],text=True)
 s=s.split('#[cfg(test)]')[0]
 s=re.sub(r'/// ```compile_fail\n.*?/// ```\n','',s,flags=re.S)
 (base/name).write_text(s)
p=base/'mod.rs';s=p.read_text()
s='pub use grid::*;\npub use shape::*;\npub use error::*;\n\nmod grid;\nmod shape;\nmod error;\nmod bindings;\nmod uniforms;\n\n'+s
s=s.replace('        workgroups_x: u32,\n        workgroups_y: u32,\n        workgroups_z: u32,','        groups: WorkgroupGrid,')
s=s.replace('            workgroups_x,\n            workgroups_y,\n            workgroups_z,','            groups,')
start=s.index('        // --- GET PIPELINE');end=s.index('        let mut bind_groups = Vec::new();',start)
s=s[:start]+'''        let pipeline = pipeline_def.compiled(context)?;
        let bind_group_layouts = pipeline_def.bind_group_layouts();
        let reflection = pipeline_def.reflection();

'''+s[end:]
# Extract existing unchanged packing match body to its own responsibility.
start=s.index('                let mut buffer_data =');end=s.index('                let buffer = context',start)
packing=s[start:end]
uniforms='use crate::data::{BindGroupReflection, PassParameter, PassParameters};\n\npub(super) fn pack_uniforms(bg_reflection: &BindGroupReflection, parameters: &PassParameters) -> Vec<u8> {\n'+packing+'\n buffer_data\n}\n'
(base/'uniforms.rs').write_text(uniforms)
s=s[:start]+'                let buffer_data = uniforms::pack_uniforms(bg_reflection, parameters);\n\n'+s[end:]
# Binding extent is a concrete native-positive boundary; logical zero itself remains valid.
start=s.index('                    let binding_resource =');end=s.index('                    bind_group_entries.push',start)
s=s[:start]+'''                    let binding_resource = gpu_buf.binding_resource(name)?;
'''+s[end:]
# Shared texture owner selects and validates effective view facts once.
start=s.index('                let (view, actual_format, dim) =');end=s.index('\n            }\n\n            for (binding, view)',start)
s=s[:start]+'''                let selected = val.texture_binding(context, binding_info)
                    .map_err(anyhow::Error::new)
                    .with_context(|| format!("ComputePass: Texture '{name}'"))?;
                texture_views.push((binding, selected.view));'''+s[end:]
# Capture concrete bind group cause at actual native producer.
start=s.index('            let bind_group = context');end=s.index('            bind_groups.push(bind_group);',start)
block=s[start:end]
block='''            #[cfg(not(target_arch = "wasm32"))]
            let scope = context.blocking_validation.then(|| context.device.push_error_scope(wgpu::ErrorFilter::Validation));
'''+block+'''            #[cfg(not(target_arch = "wasm32"))]
            if let Some(scope) = scope {
                let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
                if let Some(cause) = pollster::block_on(scope.pop()) {
                    return Err(ComputePassError::Native(std::sync::Arc::new(cause)).into());
                }
            }
'''
s=s[:start]+block+s[end:]
s=s.replace('            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, workgroups_z);','            groups.record(&mut compute_pass);')
# Respect sparse reflected indices rather than ordinal position in the populated list.
s=s.replace('            bind_groups.push(bind_group);','            bind_groups.push((bg_reflection.index, bind_group));').replace('            for (i, bg) in bind_groups.iter().enumerate() {\n                compute_pass.set_bind_group(i as u32, bg, &[]);','            for (index, bg) in &bind_groups {\n                compute_pass.set_bind_group(*index, bg, &[]);')
s=s.replace('use anyhow::{Result, anyhow};','use anyhow::{Context, Result, anyhow};')
# Extract the full bind-assembly responsibility from recording/submit.
start=s.index('        let bind_group_layouts =');end=s.index('        // Frame: Compute Pass',start)
assembly=s[start:end]
assembly=assembly.replace('        let reflection = pipeline_def.reflection();','        let reflection = pipeline_def.reflection();')
bindings='''use super::{ComputePassError, uniforms};
use crate::data::{ComputePipeline, PassParameter, PassParameters};
use crate::globals::WgpuContext;
use anyhow::{Context, Result, anyhow};
use wgpu::util::DeviceExt;

pub(super) fn create_bind_groups(context: &WgpuContext, pipeline_def: &ComputePipeline, parameters: &PassParameters) -> Result<Vec<(u32, wgpu::BindGroup)>> {
'''+assembly+'\n Ok(bind_groups)\n}\n'
(base/'bindings.rs').write_text(bindings)
s=s[:start]+'        let bind_groups = bindings::create_bind_groups(context, pipeline_def, parameters)?;\n\n'+s[end:]
s=s.replace('use anyhow::{Context, Result, anyhow};\nuse wgpu::util::DeviceExt;','use anyhow::Result;')
p.write_text(s)
