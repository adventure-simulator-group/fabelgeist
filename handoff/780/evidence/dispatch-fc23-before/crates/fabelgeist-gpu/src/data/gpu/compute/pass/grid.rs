//! A dispatched grid counts workgroups, rather than shader invocations.

/// Native per-axis folding limit used by the project's dispatch layout.
const MAX_GROUPS_PER_DIMENSION: u32 = 65_535;

/// Native x-axis workgroups, distinct from shader invocations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkgroupCount(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkgroupGrid([u32; 3]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchOccupancy {
    Empty,
    Populated,
}
impl From<u32> for WorkgroupCount {
    fn from(groups: u32) -> Self {
        Self(groups)
    }
}

impl WorkgroupCount {
    /// Native x count for one-to-one storage/launch quantity conversions.
    pub(crate) fn native_groups(self) -> u32 {
        self.0
    }
}

impl From<WorkgroupCount> for WorkgroupGrid {
    fn from(groups: WorkgroupCount) -> Self {
        Self([groups.0, 1, 1])
    }
}

impl From<[u32; 3]> for WorkgroupGrid {
    fn from(groups: [u32; 3]) -> Self {
        Self(groups)
    }
}
impl WorkgroupGrid {
    /// Admit a native x-axis group count, folding excess groups into rows.
    /// Kernels must flatten both axes and bounds-check the final row.
    pub fn folded_x(workgroups: WorkgroupCount) -> Self {
        let workgroups = workgroups.0;
        let x = workgroups.clamp(1, MAX_GROUPS_PER_DIMENSION);
        Self([x, workgroups.div_ceil(x), 1])
    }

    pub fn occupancy(self) -> DispatchOccupancy {
        if self.0.contains(&0) {
            DispatchOccupancy::Empty
        } else {
            DispatchOccupancy::Populated
        }
    }
    pub fn record(self, pass: &mut wgpu::ComputePass<'_>) {
        pass.dispatch_workgroups(self.0[0], self.0[1], self.0[2]);
    }
}
