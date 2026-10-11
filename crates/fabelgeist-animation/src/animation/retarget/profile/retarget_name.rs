//! Labels for source-to-target retargeting recipes.

use serde::{Deserialize, Serialize};

use super::RigProfileName;

/// A retargeting recipe's label, preserving every authored string spelling.
///
/// A rig profile describes how to recognize a skeleton's joints. This label
/// describes the recipe joining two rig profiles; it does not select either
/// rig, a joint or an animation clip. Empty labels, Unicode, whitespace and
/// control characters are retained. JSON, Display and Debug use the native
/// string representation.
///
/// The two-rig constructor snapshots the rig labels at construction time:
///
/// ```
/// use fabelgeist_animation::animation::retarget::{
///     RetargetProfile, RetargetProfileName, RigProfile, RigProfileName,
/// };
/// use fabelgeist_animation::Skeleton;
///
/// let source = RigProfile::new(RigProfileName::from("source"));
/// let target = RigProfile::new(RigProfileName::from("target"));
/// let name = RetargetProfileName::from_rig_labels(&source.name, &target.name);
/// let profile = RetargetProfile::new(source, target);
/// let empty = Skeleton::new(Vec::new());
/// let resolved = profile.resolve(&empty, &empty).unwrap();
/// let retained: RetargetProfileName = resolved.name;
/// assert_eq!(retained, name);
///
/// let authored = RetargetProfile {
///     name: RetargetProfileName::from(String::from("my transfer recipe")),
///     ..profile
/// };
/// assert_eq!(format!("{}", authored.name), "my transfer recipe");
/// ```
///
/// A raw String requires explicit admission at the text boundary:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::RetargetProfile;
/// let profile = RetargetProfile {
///     name: String::from("my transfer recipe"),
///     ..RetargetProfile::default()
/// };
/// ```
///
/// Rig-profile labels have a separate role:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::{RetargetProfile, RigProfileName};
/// let profile = RetargetProfile {
///     name: RigProfileName::from("my rig"),
///     ..RetargetProfile::default()
/// };
/// ```
///
/// An animation clip name also cannot stand in for the recipe label:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::AnimationClipName;
/// use fabelgeist_animation::animation::retarget::RetargetProfile;
/// let profile = RetargetProfile {
///     name: AnimationClipName::from("walk"),
///     ..RetargetProfile::default()
/// };
/// ```
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RetargetProfileName(String);

impl RetargetProfileName {
    /// Snapshot the source and target rig labels with the recipe separator.
    ///
    /// Later rig-label changes leave this label unchanged. Two empty rig
    /// labels produce `" -> "`, while the default recipe label is empty.
    pub fn from_rig_labels(source: &RigProfileName, target: &RigProfileName) -> Self {
        Self(format!("{source} -> {target}"))
    }
}

impl From<&str> for RetargetProfileName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for RetargetProfileName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl std::fmt::Display for RetargetProfileName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

impl std::fmt::Debug for RetargetProfileName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
