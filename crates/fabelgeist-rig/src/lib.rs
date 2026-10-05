//! Exact skeleton joint labels shared by asset admission and runtime consumers.
//! Anatomical roles do not replace a rig's authored joint identity.
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
mod names;
mod part;
#[cfg(test)]
mod tests;
pub use part::RigJointPart;

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RigJointName(Cow<'static, str>);

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

/// Zero-based location in an ordered rig, distinct from a parameter row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RigJointOrdinal(usize);
impl From<usize> for RigJointOrdinal {
    fn from(index: usize) -> Self {
        Self(index)
    }
}
impl From<u32> for RigJointOrdinal {
    fn from(index: u32) -> Self {
        Self(index as usize)
    }
}
impl From<RigJointOrdinal> for usize {
    fn from(index: RigJointOrdinal) -> Self {
        index.0
    }
}

/// Absence from the requested rig, retaining the exact query identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigJointLookupError {
    pub joint: RigJointName,
}
impl std::fmt::Display for RigJointLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rig joint {} is missing", self.joint)
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
        RigJointMembership::from(self.0.contains(&part.to_string()))
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
impl From<String> for RigJointName {
    fn from(name: String) -> Self {
        Self(Cow::Owned(name))
    }
}
impl From<RigJointName> for Cow<'static, str> {
    fn from(name: RigJointName) -> Self {
        name.0
    }
}
/// Borrow the original spelling at a native text-format boundary.
impl<'a> From<&'a RigJointName> for Cow<'a, str> {
    fn from(name: &'a RigJointName) -> Self {
        Cow::Borrowed(name.0.as_ref())
    }
}
impl std::fmt::Display for RigJointName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
