//! Stable strand identities and parting membership on the host.
//!
//! Integer words are admitted or extracted only by hash, file and GPU adapters.

/// A stable strand identity, independent of its current collection index.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct StrandId(u32);

/// The identity of one side or lane of a parting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PartingGroupId(u32);

/// A strand may be unassigned to a parting, rather than carrying a sentinel ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartingGroup {
    Unassigned,
    Assigned(PartingGroupId),
}

impl StrandId {
    pub const fn from_word(word: u32) -> Self {
        Self(word)
    }
    pub const fn word(self) -> u32 {
        self.0
    }
}

impl PartingGroup {
    /// Decode the shader's negative unassigned sentinel at GPU admission.
    pub const fn from_gpu_word(word: i32) -> Self {
        if word < 0 {
            Self::Unassigned
        } else {
            Self::Assigned(PartingGroupId(word as u32))
        }
    }

    /// Pack the shader's existing signed representation without layout changes.
    pub const fn gpu_word(self) -> i32 {
        match self {
            Self::Unassigned => -1,
            Self::Assigned(group) => group.0 as i32,
        }
    }

    /// Native braid integration coordinate for this parting membership.
    pub(crate) fn lane(self, lanes: u32) -> f32 {
        match self {
            Self::Assigned(group) => (group.0 % lanes) as f32,
            Self::Unassigned => (lanes - 1) as f32,
        }
    }

    pub const fn is_unassigned(self) -> bool {
        matches!(self, Self::Unassigned)
    }
}

impl Default for PartingGroup {
    fn default() -> Self {
        Self::Assigned(PartingGroupId(0))
    }
}
