from pathlib import Path
p=Path('crates/fabelgeist-compute/src/kernel/fast.rs');s=p.read_text()
# Native compilation, allocation and packing are separate responsibilities.
start=s.index('struct CompiledLayout');end=s.index('impl FastPath {',start)
chunk=s[start:end]
compile_start=chunk.index('impl CompiledLayout {');compile=chunk[:chunk.index('/// The uniform values')]+chunk[compile_start:]
compile=compile.replace('struct CompiledLayout {','pub(super) struct CompiledLayout {').replace('    layout:','    pub(super) layout:').replace('    pipeline:','    pub(super) pipeline:').replace('    fn new(','    pub(super) fn new(').replace('return Err(anyhow!("Kernel Pipeline Creation Error: {error}"));','return Err(fabelgeist_gpu::data::ComputePipelineError::Native(std::sync::Arc::new(error)).into());')
Path('crates/fabelgeist-compute/src/kernel/fast/compile.rs').write_text('use crate::prelude::*;\n\n'+compile)
s=s[:start]+s[end:]
# Extract uniform packing unchanged.
start=s.index('/// Pack the uniform block');packing=s[start:]
Path('crates/fabelgeist-compute/src/kernel/fast/uniforms.rs').write_text('use crate::prelude::*;\nuse fabelgeist_gpu::data::gpu::shader::UniformMember;\n\n'+packing.replace('fn write_uniform(','pub(super) fn write_uniform('))
s=s[:start]
s=s.replace('use fabelgeist_gpu::data::gpu::buffer::Buffer;','use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferByteLength, BufferByteOffset};\nuse std::sync::Arc;\nuse arena::{UniformArena, UniformSlot};\nuse compile::CompiledLayout;\nuse uniforms::write_uniform;\n\nmod arena;\nmod compile;\nmod uniforms;')
# Arena re-export visible to parent KernelBatch.
s=s.replace('use arena::{UniformArena, UniformSlot};','pub(super) use arena::UniformArena;')
s=s.replace('const CHUNK_BYTES: u64 = 64 * 1024;\n\n','')
s=s.replace('    uniform_size: u64,','    uniform_size: BufferByteLength,').replace('    stride: u64,','    stride: BufferByteLength,\n    device: Arc<wgpu::Device>,')
# Recreate Prepared with native offset extraction only at pass boundary.
pos=s.index('impl FastPath {')
s=s[:pos]+'''/// Native dispatch resources admitted for one batch recording.
pub(super) struct Prepared {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    dynamic_offset: Option<BufferByteOffset>,
}

impl Prepared {
    pub(super) fn record(&self, pass: &mut wgpu::ComputePass<'_>, groups: WorkgroupGrid) -> Result<()> {
        let offset = self.dynamic_offset.map(|offset| u32::try_from(u64::from(offset)))
            .transpose().map_err(|_| arena::UniformArenaError::DynamicOffsetOverflow)?;
        pass.set_pipeline(&self.pipeline);
        match offset {
            Some(offset) => pass.set_bind_group(0, &self.bind_group, &[offset]),
            None => pass.set_bind_group(0, &self.bind_group, &[]),
        }
        groups.record(pass);
        Ok(())
    }
}

'''+s[pos:]
s=s.replace('        let uniform_size = group.uniform_buffer_size as u64;','        let uniform_size = u64::from(group.uniform_buffer_size);')
s=s.replace('            uniform_size,\n            stride,','            uniform_size: uniform_size.into(),\n            stride: stride.into(),\n            device: context.device.clone(),')
start=s.index('        let slot = (self.uniform_binding');end=s.index('        // Built fresh',start)
s=s[:start]+'''        if self.device.as_ref() != context.device.as_ref() {
            return Err(ComputePipelineError::WrongDevice.into());
        }
'''+s[end:]
start=s.index('                resource: if u64::from(buffer.size)');end=s.index('\n            });',start)
s=s[:start]+'''                resource: buffer.binding_resource(&binding.to_string())?,'''+s[end:]
# Stage all parameter packing before committing a slot or creating a native binding.
start=s.index('        if let (Some(binding), Some((chunk, _)))');end=s.index('        entries.sort_by_key',start)
s=s[:start]+'''        let mut data = [0u8; MAX_UNIFORM_BYTES];
        let size = usize::try_from(u64::from(self.uniform_size))
            .map_err(|_| arena::UniformArenaError::AddressOverflow)?;
        let data = data.get_mut(..size).ok_or(arena::UniformArenaError::StrideExceedsChunk)?;
        if self.uniform_binding.is_some() {
            write_uniform(&self.uniform_members, parameters, data)?;
        }
        let slot = self.uniform_binding.map(|_| uniforms.allocate(context, self.stride)).transpose()?;
        if let (Some(binding), Some(slot)) = (self.uniform_binding, &slot) {
            entries.push(wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &slot.buffer, offset: 0,
                    size: std::num::NonZeroU64::new(u64::from(self.uniform_size)),
                }),
            });
        }
'''+s[end:]
start=s.index('        let bind_group = context')
s=s[:start]+'''        #[cfg(not(target_arch = "wasm32"))]
        let scope = context.blocking_validation.then(|| context.device.push_error_scope(wgpu::ErrorFilter::Validation));
'''+s[start:]
start=s.index('        let dynamic_offset = match &slot');end=s.index('\n        Ok(Prepared',start)
s=s[:start]+'''        let dynamic_offset = slot.as_ref().map(|slot| {
            context.queue.write_buffer(&slot.buffer, u64::from(slot.offset), data);
            slot.offset
        });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ComputePassError::Native(Arc::new(cause)).into());
            }
        }
'''+s[end:]
p.write_text(s)
