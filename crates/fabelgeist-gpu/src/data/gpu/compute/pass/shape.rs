//! Declared invocation dimensions and their covering dispatch grid.
use std::num::NonZeroU32;

use super::{WorkgroupCount, WorkgroupGrid};

/// Invocations to cover along x, distinct from a count of workgroups.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvocationCount(u32);

/// A shader's admitted, nonzero invocation dimensions within one workgroup.
///
/// Device limits remain the pipeline provider's admission responsibility.
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{InvocationCount, WorkgroupGrid, WorkgroupShape};
/// let shape = WorkgroupShape::try_from([64, 1, 1]).unwrap();
/// shape.covering_x(WorkgroupGrid::from([1, 1, 1]));
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WorkgroupShape([NonZeroU32; 3]);

/// A native declaration with at least one zero invocation dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkgroupShapeError([u32; 3]);

impl From<u32> for InvocationCount {
    fn from(items: u32) -> Self {
        Self(items)
    }
}

impl std::fmt::Display for WorkgroupShapeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workgroup size {:?} has a zero dimension",
            self.0
        )
    }
}
impl std::error::Error for WorkgroupShapeError {}
impl TryFrom<[u32; 3]> for WorkgroupShape {
    type Error = WorkgroupShapeError;
    fn try_from(axes: [u32; 3]) -> Result<Self, WorkgroupShapeError> {
        let [Some(x), Some(y), Some(z)] = axes.map(NonZeroU32::new) else {
            return Err(WorkgroupShapeError(axes));
        };
        Ok(Self([x, y, z]))
    }
}
impl WorkgroupShape {
    /// Cover native resource axes with the admitted declaration's dimensions.
    /// The axes count invocations; the returned axes count workgroups.
    pub fn covering(self, axes: [InvocationCount; 3]) -> WorkgroupGrid {
        let [x, y, z] = axes;
        WorkgroupGrid::from([
            x.0.div_ceil(self.0[0].get()),
            y.0.div_ceil(self.0[1].get()),
            z.0.div_ceil(self.0[2].get()),
        ])
    }

    /// Cover invocations with rows of workgroups for a folded-index kernel.
    pub fn covering_folded_x(self, items: InvocationCount) -> WorkgroupGrid {
        WorkgroupGrid::folded_x(self.workgroup_count_x(items))
    }

    /// X-axis workgroups needed to cover an invocation prefix.
    pub fn workgroup_count_x(self, items: InvocationCount) -> WorkgroupCount {
        WorkgroupCount::from(items.0.div_ceil(self.0[0].get()))
    }

    /// Cover items along x with ceiling division. Y and z each dispatch one
    /// group, retaining the kernel API's existing one-dimensional policy.
    pub fn covering_x(self, items: InvocationCount) -> WorkgroupGrid {
        self.workgroup_count_x(items).into()
    }
}
impl std::fmt::Debug for WorkgroupShape {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.map(NonZeroU32::get).fmt(formatter)
    }
}

#[cfg(test)]
mod tests;
