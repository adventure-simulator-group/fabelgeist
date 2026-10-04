//! Display labels for a rig and for a transfer between rigs.
use serde::{Deserialize, Serialize};

/// A profile label does not acquire skeleton-joint lookup authority.
///
/// ```compile_fail
/// use fabelgeist_animation::animation::retarget::{RigProfile, RetargetProfileName};
/// RigProfile::new(RetargetProfileName::from("source -> target"));
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Debug for RigProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

/// The display identity of a source-to-target transfer recipe.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RetargetProfileName(String);
impl From<(&RigProfileName, &RigProfileName)> for RetargetProfileName {
    fn from((source, target): (&RigProfileName, &RigProfileName)) -> Self {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Debug for RetargetProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
