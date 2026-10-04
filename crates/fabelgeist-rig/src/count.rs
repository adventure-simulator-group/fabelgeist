//! Cardinality and traversal of an ordered skeleton, including an empty rig.
use super::RigJointOrdinal;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RigJointCount(usize);
impl From<usize> for RigJointCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
impl From<RigJointCount> for usize {
    fn from(count: RigJointCount) -> Self {
        count.0
    }
}
impl std::fmt::Display for RigJointCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl RigJointCount {
    pub fn ordinals(self) -> RigJointOrdinals {
        RigJointOrdinals(0..self.0)
    }
}
pub struct RigJointOrdinals(std::ops::Range<usize>);
impl Iterator for RigJointOrdinals {
    type Item = RigJointOrdinal;
    fn next(&mut self) -> Option<RigJointOrdinal> {
        self.0.next().map(RigJointOrdinal::from)
    }
}
