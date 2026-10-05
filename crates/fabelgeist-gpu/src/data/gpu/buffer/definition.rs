//! Buffer capability selection and its native allocation descriptor.

use crate::globals::WgpuContext;

use super::{BufferByteLength, BufferLabel};

/// Independent uses of one allocation; storage also permits device copies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUse {
    Uniform,
    Storage,
    Vertex,
    Index,
    Indirect,
    CopySource,
    CopyDestination,
    HostWrite,
    HostRead,
}

#[derive(Clone, Copy, Debug)]
enum UsageSelection {
    Unspecified,
    Explicit(wgpu::BufferUsages),
}

#[derive(Clone, Debug)]
pub struct BufferDefinition {
    label: Option<BufferLabel>,
    usage: UsageSelection,
}

const GENERAL_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::UNIFORM)
    .union(wgpu::BufferUsages::VERTEX)
    .union(wgpu::BufferUsages::INDEX)
    .union(wgpu::BufferUsages::INDIRECT)
    .union(wgpu::BufferUsages::COPY_DST)
    .union(wgpu::BufferUsages::COPY_SRC);

impl Default for BufferDefinition {
    fn default() -> Self {
        Self::all()
    }
}

impl BufferDefinition {
    pub fn new() -> Self {
        Self {
            label: None,
            usage: UsageSelection::Unspecified,
        }
    }

    pub fn all() -> Self {
        Self {
            label: None,
            usage: UsageSelection::Explicit(GENERAL_USAGE),
        }
    }

    pub fn with_label(mut self, label: BufferLabel) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_usage(mut self, selected: BufferUse) -> Self {
        let flags = match selected {
            BufferUse::Uniform => wgpu::BufferUsages::UNIFORM,
            BufferUse::Storage => {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
            }
            BufferUse::Vertex => wgpu::BufferUsages::VERTEX,
            BufferUse::Index => wgpu::BufferUsages::INDEX,
            BufferUse::Indirect => wgpu::BufferUsages::INDIRECT,
            BufferUse::CopySource => wgpu::BufferUsages::COPY_SRC,
            BufferUse::CopyDestination => wgpu::BufferUsages::COPY_DST,
            BufferUse::HostWrite => wgpu::BufferUsages::MAP_WRITE,
            BufferUse::HostRead => wgpu::BufferUsages::MAP_READ,
        };
        let existing = match self.usage {
            UsageSelection::Unspecified => wgpu::BufferUsages::empty(),
            UsageSelection::Explicit(flags) => flags,
        };
        self.usage = UsageSelection::Explicit(existing | flags);
        self
    }

    pub fn uniform() -> Self {
        Self::new().with_usage(BufferUse::Uniform)
    }
    pub fn storage() -> Self {
        Self::new().with_usage(BufferUse::Storage)
    }
    pub fn vertex() -> Self {
        Self::new().with_usage(BufferUse::Vertex)
    }
    pub fn index() -> Self {
        Self::new().with_usage(BufferUse::Index)
    }
    pub fn indirect() -> Self {
        Self::new().with_usage(BufferUse::Indirect)
    }
    pub fn map_write() -> Self {
        Self::new().with_usage(BufferUse::HostWrite)
    }
    pub fn map_read() -> Self {
        Self::new().with_usage(BufferUse::HostRead)
    }
    pub fn copy_src() -> Self {
        Self::new().with_usage(BufferUse::CopySource)
    }
    pub fn copy_dst() -> Self {
        Self::new().with_usage(BufferUse::CopyDestination)
    }

    pub(super) fn allocate_native(
        &self,
        context: &WgpuContext,
        length: BufferByteLength,
    ) -> wgpu::Buffer {
        let usage = match self.usage {
            UsageSelection::Unspecified => GENERAL_USAGE,
            UsageSelection::Explicit(flags) => flags,
        };
        context.device.create_buffer(&wgpu::BufferDescriptor {
            label: self.label.as_ref().map(<&str>::from),
            size: u64::from(length),
            usage,
            mapped_at_creation: false,
        })
    }
}

#[cfg(test)]
mod tests;
