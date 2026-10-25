//! Declared invocation dimensions and their covering dispatch grid.
use std::num::NonZeroU32;

use super::{DispatchOccupancy, WorkgroupCount, WorkgroupGrid};

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

/// A declaration or padded launch that cannot use a u32 folded-X index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldedDispatchError {
    NonLinearShape {
        shape: WorkgroupShape,
    },
    AddressOverflow {
        shape: WorkgroupShape,
        groups: WorkgroupGrid,
    },
}

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

    /// Cover invocations with rows of one-dimensional workgroups.
    ///
    /// The kernel flattens `id.x + id.y * groups.x * width` into a u32
    /// index and bounds-checks it against its live population. Admit the
    /// padded final row too: wrapping a padded index could pass that check
    /// and revisit live data. Empty grids launch no indexing work. Device
    /// limits are checked by the receiver.
    pub fn covering_folded_x(
        self,
        items: InvocationCount,
    ) -> Result<WorkgroupGrid, FoldedDispatchError> {
        let groups = WorkgroupGrid::folded_x(self.workgroup_count_x(items));
        if groups.occupancy() == DispatchOccupancy::Empty {
            return Ok(groups);
        }
        if self.0[1].get() != 1 || self.0[2].get() != 1 {
            return Err(FoldedDispatchError::NonLinearShape { shape: self });
        }
        let [x, y, _] = groups.dimensions();
        let covered = u64::from(x)
            .checked_mul(u64::from(y))
            .and_then(|rows| rows.checked_mul(u64::from(self.0[0].get())));
        if covered.is_none_or(|covered| covered > u64::from(u32::MAX) + 1) {
            return Err(FoldedDispatchError::AddressOverflow {
                shape: self,
                groups,
            });
        }
        Ok(groups)
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

impl std::fmt::Display for FoldedDispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonLinearShape { shape } => write!(
                formatter,
                "folded-X dispatch requires a one-dimensional workgroup, received {shape:?}"
            ),
            Self::AddressOverflow { shape, groups } => write!(
                formatter,
                "folded dispatch {groups:?} of shape {shape:?} exceeds u32 lane addresses"
            ),
        }
    }
}

impl std::error::Error for FoldedDispatchError {}

#[cfg(test)]
mod tests;
