//! Spelling at the boundary of the canonical humanoid role vocabulary.

use std::borrow::Cow;

/// A potential humanoid role spelling, distinct from an authored joint's name.
///
/// Text is preserved exactly, including empty and unknown names. Classification
/// into [`super::HumanoidJoint`] checks membership in the role catalogue.
///
/// ```
/// use fabelgeist_animation::animation::retarget::{HumanoidJoint, HumanoidRoleName};
/// let name = HumanoidRoleName::from("UpperArmLeft");
/// assert_eq!(HumanoidJoint::try_from(name), Ok(HumanoidJoint::UpperArmLeft));
/// ```
///
/// Classification accepts the bespoke spelling type directly:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::HumanoidJoint;
/// let role = HumanoidJoint::try_from(String::from("Pelvis"));
/// ```
///
/// A rig-profile label belongs to a different vocabulary:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::{HumanoidJoint, RigProfileName};
/// let role = HumanoidJoint::try_from(RigProfileName::from("Pelvis"));
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct HumanoidRoleName(pub(super) Cow<'static, str>);

impl HumanoidRoleName {
    pub(super) const fn from_static(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }
}

impl From<&str> for HumanoidRoleName {
    fn from(name: &str) -> Self {
        Self(Cow::Owned(name.to_owned()))
    }
}

impl From<String> for HumanoidRoleName {
    fn from(name: String) -> Self {
        Self(Cow::Owned(name))
    }
}

impl std::fmt::Debug for HumanoidRoleName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}

impl std::fmt::Display for HumanoidRoleName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}
