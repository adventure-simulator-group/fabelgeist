//! A dispatched grid counts workgroups, rather than shader invocations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkgroupGrid([u32; 3]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchOccupancy {
    Empty,
    Populated,
}
impl From<[u32; 3]> for WorkgroupGrid {
    fn from(groups: [u32; 3]) -> Self {
        Self(groups)
    }
}
impl WorkgroupGrid {
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
