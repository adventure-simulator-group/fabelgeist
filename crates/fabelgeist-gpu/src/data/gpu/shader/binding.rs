//! Addresses in a shader's resource layout.
use std::fmt;
use std::sync::Arc;

/// A resource group coordinate, distinct from a binding within that group.
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{BindGroupIndex, BindingIndex};
/// let group: BindGroupIndex = BindingIndex::from(0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindGroupIndex(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingIndex(u32);

impl BindGroupIndex {
    pub const FIRST: Self = Self(0);

    /// The native layout table is addressed by the declared group coordinate.
    pub fn layout(
        self,
        layouts: &[Arc<wgpu::BindGroupLayout>],
    ) -> Option<&Arc<wgpu::BindGroupLayout>> {
        layouts.get(self.0 as usize)
    }
}
impl From<u32> for BindGroupIndex {
    fn from(index: u32) -> Self {
        Self(index)
    }
}
impl From<BindGroupIndex> for u32 {
    fn from(index: BindGroupIndex) -> Self {
        index.0
    }
}
impl From<u32> for BindingIndex {
    fn from(index: u32) -> Self {
        Self(index)
    }
}
impl From<BindingIndex> for u32 {
    fn from(index: BindingIndex) -> Self {
        index.0
    }
}
impl fmt::Display for BindGroupIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
impl fmt::Display for BindingIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
