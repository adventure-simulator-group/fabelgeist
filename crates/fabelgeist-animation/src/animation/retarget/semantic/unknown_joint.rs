//! Rejection at the humanoid role spelling boundary.

use super::HumanoidRoleName;

/// A spelling that does not identify a canonical humanoid role.
///
/// The rejected bespoke name remains available without branching on diagnostic
/// prose. Display and Debug retain the native String error's formatting.
///
/// ```
/// use fabelgeist_animation::animation::retarget::{HumanoidJoint, HumanoidRoleName};
/// let name = HumanoidRoleName::from("pelvis");
/// let error = HumanoidJoint::try_from(name.clone()).unwrap_err();
/// assert_eq!(error.name(), &name);
/// assert_eq!(error.to_string(), "unknown humanoid joint \"pelvis\"");
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct UnknownHumanoidJoint {
    name: HumanoidRoleName,
}

impl UnknownHumanoidJoint {
    pub(super) fn from_rejected_name(name: HumanoidRoleName) -> Self {
        Self { name }
    }

    /// The exact spelling rejected by the role catalogue.
    pub fn name(&self) -> &HumanoidRoleName {
        &self.name
    }
}

impl std::fmt::Debug for UnknownHumanoidJoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.to_string(), formatter)
    }
}

impl std::fmt::Display for UnknownHumanoidJoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Rendering is the native formatting boundary; the cause stays typed.
        std::fmt::Display::fmt(
            &format!("unknown humanoid joint {:?}", self.name),
            formatter,
        )
    }
}

impl std::error::Error for UnknownHumanoidJoint {}
