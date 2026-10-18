//! Original constraint addresses remain separate from color-order slots.
use fabelgeist_gpu::prelude::{BufferByteLength, PassParameter};

/// Address of a constraint in its producer's original order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintIndex(u32);

/// Complete per-constraint record cardinality, including an empty set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstraintCount(usize);

impl From<u32> for ConstraintIndex {
    fn from(word: u32) -> Self {
        Self(word)
    }
}

impl From<ConstraintIndex> for usize {
    fn from(index: ConstraintIndex) -> Self {
        index.0 as usize
    }
}

impl std::fmt::Display for ConstraintIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl From<usize> for ConstraintCount {
    fn from(records: usize) -> Self {
        Self(records)
    }
}

impl From<ConstraintCount> for usize {
    fn from(count: ConstraintCount) -> Self {
        count.0
    }
}

impl std::fmt::Display for ConstraintCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl ConstraintCount {
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub(crate) fn lambda_bytes(self) -> BufferByteLength {
        ((self.0.max(1) as u64) * 4).into()
    }
    pub(crate) fn uniform(self) -> PassParameter {
        (self.0 as u32).into()
    }
    pub(crate) fn dispatch_items(self) -> u32 {
        self.0 as u32
    }
}
