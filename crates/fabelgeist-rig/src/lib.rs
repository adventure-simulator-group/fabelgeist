//! Exact skeleton joint labels shared by asset admission and runtime consumers.
//! Anatomical roles do not replace a rig's authored joint identity.

use derive_more::{Display, From, Into};
pub use part::RigJointPart;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, num::TryFromIntError};

mod names;
mod part;

/// An exact rig label. Namespace policy belongs to its source format.
/// Empty and unfamiliar labels are retained; identity alone does not prove
/// membership in a skeleton. Serialization is the original scalar string.
///
/// ```compile_fail
/// use fabelgeist_rig::{RigJointName, RigJointPart, RigSide};
/// RigJointName::sided(RigSide::Left, RigJointName::L_UPARM);
/// ```
///
/// ```compile_fail
/// use fabelgeist_rig::{RigJointName, RigJointOrdinal};
/// let slot: RigJointOrdinal = RigJointName::ROOT;
/// ```
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display, From, Into,
)]
#[serde(transparent)]
#[from(String)]
pub struct RigJointName(Cow<'static, str>);

/// Zero-based location in an ordered rig, distinct from a parameter row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct RigJointOrdinal(usize);

/// Absence from the requested rig, retaining the exact query identity.
#[derive(Clone, Debug, PartialEq, Eq, Display)]
#[display("rig joint {joint} is missing")]
pub struct RigJointLookupError {
    pub joint: RigJointName,
}

/// Side in an authored rig label, independent of equipment placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RigSide {
    Left,
    Right,
}

/// Result of a semantic skin-name query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RigJointMembership {
    Included,
    Excluded,
}

impl From<bool> for RigJointMembership {
    fn from(included: bool) -> Self {
        if included {
            Self::Included
        } else {
            Self::Excluded
        }
    }
}

impl From<RigJointMembership> for u32 {
    fn from(membership: RigJointMembership) -> Self {
        match membership {
            RigJointMembership::Included => 1,
            RigJointMembership::Excluded => 0,
        }
    }
}

impl From<u32> for RigJointOrdinal {
    fn from(index: u32) -> Self {
        Self(index as usize)
    }
}

/// Checked encoding for native formats whose joint words are 32 bits.
impl TryFrom<RigJointOrdinal> for u32 {
    type Error = TryFromIntError;

    fn try_from(index: RigJointOrdinal) -> Result<Self, Self::Error> {
        Self::try_from(index.0)
    }
}

impl std::error::Error for RigJointLookupError {}

impl RigJointName {
    /// A query-specific comparison; exact identity remains case-sensitive.
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0.as_ref())
    }

    /// Authored MHR labels include world/pelvis and center or sided families.
    pub fn mhr_membership(&self) -> RigJointMembership {
        RigJointMembership::from(
            self.mhr_center_membership() == RigJointMembership::Included
                || self.0.starts_with("l_")
                || self.0.starts_with("r_"),
        )
    }

    /// Center joints are retained independently of mirrored left/right pairs.
    pub fn mhr_center_membership(&self) -> RigJointMembership {
        RigJointMembership::from(
            self == &Self::BODY_WORLD || self == &Self::ROOT || self.0.starts_with("c_"),
        )
    }

    /// The exact right-side label corresponding to an authored left label.
    /// Existence in the receiving rig is checked separately.
    pub fn mhr_right_partner(&self) -> Option<Self> {
        self.0
            .strip_prefix("l_")
            .map(|suffix| Self::from(format!("r_{suffix}")))
    }

    pub fn sided(side: RigSide, part: RigJointPart) -> Self {
        let prefix = match side {
            RigSide::Left => "l",
            RigSide::Right => "r",
        };
        Self(Cow::Owned(format!("{prefix}_{part}")))
    }

    /// Exact owner and every spelling with its `_twist` prefix.
    /// Suffixes intentionally remain unconstrained, matching authored skin policy.
    pub fn skin_family(&self, owner: &Self) -> RigJointMembership {
        RigJointMembership::from(self == owner || self.0.starts_with(&format!("{}_twist", owner.0)))
    }

    /// Authored joint families such as all thumb or tongue segments.
    pub fn family_prefix(&self, prefix: &Self) -> RigJointMembership {
        RigJointMembership::from(self.0.starts_with(prefix.0.as_ref()))
    }

    /// Existing garment selection matches anatomical fragments anywhere in
    /// the label, including helper suffixes. It does not assert canonical anatomy.
    pub fn contains_part(&self, part: RigJointPart) -> RigJointMembership {
        RigJointMembership::from(self.0.contains(part.as_ref()))
    }

    /// Animation masks match an underscore-prefixed anatomical fragment in
    /// lowercase spelling, including helper suffixes and namespaced labels.
    /// This query does not normalize the retained authored identity.
    pub fn animation_part(&self, part: RigJointPart) -> RigJointMembership {
        RigJointMembership::from(self.0.to_ascii_lowercase().contains(&format!("_{part}")))
    }

    /// Preserve the first occurrence when a rig contains duplicate labels.
    pub fn index_in(&self, names: &[Self]) -> Option<RigJointOrdinal> {
        names
            .iter()
            .position(|name: &Self| -> bool { name == self })
            .map(RigJointOrdinal::from)
    }
    pub fn require_in(&self, names: &[Self]) -> Result<RigJointOrdinal, RigJointLookupError> {
        self.index_in(names).ok_or_else(|| -> RigJointLookupError {
            RigJointLookupError {
                joint: self.clone(),
            }
        })
    }
}

impl From<&str> for RigJointName {
    fn from(name: &str) -> Self {
        Self(Cow::Owned(name.to_owned()))
    }
}

/// Borrow the original spelling at a native text-format boundary.
impl<'a> From<&'a RigJointName> for Cow<'a, str> {
    fn from(name: &'a RigJointName) -> Self {
        Cow::Borrowed(name.0.as_ref())
    }
}

#[cfg(test)]
mod tests;
