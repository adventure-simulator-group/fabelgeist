//! Labels for rig profiles, independent of skeleton joint names.

use serde::{Deserialize, Serialize};

/// A rig-profile label, preserving every authored string spelling.
///
/// Empty names, Unicode, whitespace and control characters are retained. This
/// label does not establish skeleton identity, uniqueness or joint membership.
/// JSON remains a string; Display and Debug use the native string formatting.
///
/// Profile construction accepts the label directly:
///
/// ```
/// use fabelgeist_animation::animation::retarget::{RigProfile, RigProfileName};
/// use fabelgeist_animation::Skeleton;
///
/// let name = RigProfileName::from("my rig");
/// let profile = RigProfile::new(name.clone());
/// let resolved = profile.resolve(&Skeleton::new(Vec::new())).unwrap();
/// let retained: RigProfileName = resolved.profile;
/// assert_eq!(retained, name);
/// ```
///
/// A raw String requires explicit admission before profile construction:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::RigProfile;
/// RigProfile::new(String::from("my rig"));
/// ```
///
/// Direct field assignment requires the same label role:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::RigProfile;
/// let mut profile = RigProfile::default();
/// profile.name = String::from("my rig");
/// ```
///
/// A clip name cannot stand in for a rig-profile label:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::AnimationClipName;
/// use fabelgeist_animation::animation::retarget::RigProfile;
/// RigProfile::new(AnimationClipName::from("walk"));
/// ```
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RigProfileName(String);

impl From<&str> for RigProfileName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for RigProfileName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl std::fmt::Display for RigProfileName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

impl std::fmt::Debug for RigProfileName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
