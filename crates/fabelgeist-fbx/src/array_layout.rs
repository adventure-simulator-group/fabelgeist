//! Declared array cardinality and the five binary element layouts.

/// An unsigned encoded element count, distinct from a payload byte length.
/// All wire declarations, including zero, remain admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FbxArrayCount(u32);

impl From<u32> for FbxArrayCount {
    fn from(count: u32) -> Self {
        Self(count)
    }
}

impl FbxArrayCount {
    /// Native iteration bound for the decoded element buffer.
    pub(super) fn native_len(self) -> usize {
        self.0 as usize
    }

    /// Native allocation/length-check adapter, preserving host multiplication.
    /// This does not add an overflow check or resource limit.
    pub(super) fn native_byte_len(self, kind: FbxArrayKind) -> usize {
        self.native_len() * kind.native_width()
    }
}

/// The element layouts supported by binary FBX array properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FbxArrayKind {
    Float32,
    Float64,
    Integer32,
    Integer64,
    Boolean,
}

impl FbxArrayKind {
    /// Byte stride at native buffer and diagnostic adapters.
    pub(super) fn native_width(self) -> usize {
        match self {
            Self::Boolean => 1,
            Self::Float32 | Self::Integer32 => 4,
            Self::Float64 | Self::Integer64 => 8,
        }
    }
}
