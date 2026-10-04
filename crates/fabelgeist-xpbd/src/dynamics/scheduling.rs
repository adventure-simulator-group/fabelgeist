//! Frame subdivision and constraint sweeps own distinct scheduling roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubstepCount(pub(super) u32);
impl From<u32> for SubstepCount {
    fn from(substeps: u32) -> Self {
        Self(substeps)
    }
}
impl From<SubstepCount> for u32 {
    fn from(count: SubstepCount) -> Self {
        count.0
    }
}
impl SubstepCount {
    pub const DEFAULT: Self = Self(10);
    pub const EMPTY: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub fn at_least_one(self) -> Self {
        Self(self.0.max(1))
    }
    pub fn cadence(self) -> SubstepCadence {
        match self.0 {
            0 => SubstepCadence::Disabled,
            1 => SubstepCadence::Single,
            _ => SubstepCadence::Multiple,
        }
    }
    pub fn sequence(self) -> Substeps {
        Substeps {
            next: SubstepIndex::FIRST,
            count: self,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubstepCadence {
    Disabled,
    Single,
    Multiple,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubstepIndex(u32);
impl SubstepIndex {
    pub const FIRST: Self = Self(0);

    pub fn starts_cycle(self, interval: SubstepCount) -> SubstepCycleBoundary {
        if interval.0 > 0 && self.0.is_multiple_of(interval.0) {
            SubstepCycleBoundary::Boundary
        } else {
            SubstepCycleBoundary::Interior
        }
    }
    pub fn ends_cycle(self, interval: SubstepCount, total: SubstepCount) -> SubstepCycleBoundary {
        if interval.0 > 0 && ((self.0 + 1).is_multiple_of(interval.0) || self.0 + 1 == total.0) {
            SubstepCycleBoundary::Boundary
        } else {
            SubstepCycleBoundary::Interior
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubstepCycleBoundary {
    Boundary,
    Interior,
}
pub struct Substeps {
    next: SubstepIndex,
    count: SubstepCount,
}
impl Iterator for Substeps {
    type Item = SubstepIndex;
    fn next(&mut self) -> Option<SubstepIndex> {
        if self.next.0 >= self.count.0 {
            return None;
        }
        let index = self.next;
        self.next.0 += 1;
        Some(index)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintSweepCount(u32);
impl From<u32> for ConstraintSweepCount {
    fn from(sweeps: u32) -> Self {
        Self(sweeps)
    }
}
impl ConstraintSweepCount {
    pub const ONE: Self = Self(1);

    pub fn sequence(self) -> ConstraintSweeps {
        ConstraintSweeps {
            remaining: Self(self.0.max(1)),
        }
    }
}
pub struct ConstraintSweeps {
    remaining: ConstraintSweepCount,
}
impl Iterator for ConstraintSweeps {
    type Item = ();
    fn next(&mut self) -> Option<()> {
        if self.remaining.0 == 0 {
            return None;
        }
        self.remaining.0 -= 1;
        Some(())
    }
}
