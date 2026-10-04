//! A clip's authored display identity, separate from its target joint labels.
use serde::{Deserialize, Serialize};

/// A clip identity cannot be supplied as a joint query.
///
/// ```compile_fail
/// use fabelgeist_animation::animation::{Animation, AnimationClipName};
/// Animation::new("walk".into()).track(&AnimationClipName::from("hips"));
/// ```
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnimationClipName(String);
impl From<&str> for AnimationClipName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
impl From<String> for AnimationClipName {
    fn from(name: String) -> Self {
        Self(name)
    }
}
impl std::fmt::Display for AnimationClipName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Debug for AnimationClipName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
