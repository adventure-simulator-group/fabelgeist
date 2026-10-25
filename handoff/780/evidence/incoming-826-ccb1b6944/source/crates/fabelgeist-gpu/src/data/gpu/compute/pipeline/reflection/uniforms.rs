//! Retain the native byte layout and immediate struct-member lookup names.

use super::GroupBuilder;
use crate::data::gpu::shader::UniformMember;
use crate::data::{
    BufferByteLength, BufferByteOffset, ComputePipelineError, ComputePipelineResult,
};
use wgpu::naga::{Handle, Module, Type, TypeInner};

impl GroupBuilder {
    pub(super) fn uniform(
        &mut self,
        module: &Module,
        ty: Handle<Type>,
        binding: u32,
    ) -> ComputePipelineResult<()> {
        if let Some(first) = self.reflection.uniform_binding {
            return Err(ComputePipelineError::MultipleUniformBindings {
                group: self.reflection.index,
                first,
                additional: binding,
            });
        }
        let ty = &module.types[ty].inner;
        let mut reflected = Vec::new();
        let span = if let TypeInner::Struct { members, span } = ty {
            for member in members {
                let name = member.name.clone().unwrap_or_default();
                let ty = &module.types[member.ty].inner;
                reflected.push(UniformMember {
                    name,
                    offset: BufferByteOffset::from(u64::from(member.offset)),
                    size: BufferByteLength::from(u64::from(ty.size(module.to_ctx()))),
                });
            }
            *span
        } else {
            // Only immediate struct members are named parameter destinations.
            // Direct nonstruct blocks retain their existing zero initialization.
            ty.size(module.to_ctx())
        };
        self.reflection.uniform_binding = Some(binding);
        self.reflection.uniform_buffer_size = BufferByteLength::from(u64::from(span));
        self.reflection.uniform_members = reflected;
        self.entry(
            binding,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        );
        Ok(())
    }
}
