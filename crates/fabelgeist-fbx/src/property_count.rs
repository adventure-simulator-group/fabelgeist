//! Declared property cardinality admitted from a node-header word.

/// The unsigned wire declaration, distinct from offsets and byte lengths.
/// Zero and all encoded words remain admitted; resource limits are separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FbxPropertyCount(u64);

impl From<u64> for FbxPropertyCount {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl FbxPropertyCount {
    /// Length for native Vec capacity and reader iteration only.
    /// Retains the reader's existing host-sized conversion without a new bound.
    pub(super) fn native_len(self) -> usize {
        self.0 as usize
    }
}
