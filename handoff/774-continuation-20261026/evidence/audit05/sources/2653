//! Original constraint addresses remain separate from color-order slots.
use derive_more::{Display, From, Into};
use fabelgeist_gpu::prelude::{BufferByteLength, PassParameter};

/// Address of a constraint in its producer's original order.
#[derive(Clone, Copy, Debug, Display, From, PartialEq, Eq)]
pub struct ConstraintIndex(u32);

/// Complete per-constraint record cardinality, including an empty set.
#[derive(Clone, Copy, Debug, Display, From, Into, PartialEq, Eq)]
pub struct ConstraintCount(usize);

impl From<ConstraintIndex> for usize {
    fn from(index: ConstraintIndex) -> Self {
        index.0 as usize
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
