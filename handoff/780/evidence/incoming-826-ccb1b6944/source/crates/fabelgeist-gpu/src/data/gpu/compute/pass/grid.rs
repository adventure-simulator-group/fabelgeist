//! A dispatched grid counts workgroups, rather than shader invocations.

use std::{error::Error, fmt};

/// Native per-axis folding limit used by the project's dispatch layout.
const MAX_GROUPS_PER_DIMENSION: u32 = 65_535;

pub type WorkgroupGridResult<T> = Result<T, WorkgroupGridError>;

/// Native x-axis workgroups, distinct from shader invocations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkgroupCount(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkgroupGrid([u32; 3]);

/// A native dispatch exceeds the receiving device's per-axis group limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkgroupGridError {
    pub requested: WorkgroupGrid,
    pub limit: WorkgroupCount,
}

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
    /// This constructs geometry only. An unguarded lane-flattening kernel
    /// admits its declaration and padded addresses through
    /// [`super::WorkgroupShape::covering_folded_x`]. A block-index kernel
    /// can instead guard the block ordinal before computing lane addresses.
    /// The receiver checks actual device limits for either route.
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

    /// Check all native axes against the actual receiving device.
    /// Other pipeline, resource and submission validation remains separate.
    pub fn validate_device(self, device: &wgpu::Device) -> WorkgroupGridResult<Self> {
        let limit = device.limits().max_compute_workgroups_per_dimension;
        if self.0.into_iter().any(|groups| groups > limit) {
            return Err(WorkgroupGridError {
                requested: self,
                limit: WorkgroupCount::from(limit),
            });
        }
        Ok(self)
    }

    /// Record native axes after the caller's device and shader admission.
    /// This adapter has no device or declaration from which to repeat it.
    pub fn record(self, pass: &mut wgpu::ComputePass<'_>) {
        pass.dispatch_workgroups(self.0[0], self.0[1], self.0[2]);
    }

    /// Native axes for the owning shape's padded-address admission.
    pub(super) fn dimensions(self) -> [u32; 3] {
        self.0
    }
}

impl fmt::Display for WorkgroupGridError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "dispatch grid {:?} exceeds device per-axis limit {}",
            self.requested.0, self.limit.0
        )
    }
}

impl Error for WorkgroupGridError {}
