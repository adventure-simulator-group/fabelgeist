//! A clip's authored identity, separate from the joints its tracks address.

use serde::{Deserialize, Serialize};

/// An animation clip name, preserving every authored string spelling.
///
/// The native string is admitted at construction and serialization boundaries.
/// Empty names, Unicode, whitespace, and control characters are retained. JSON
/// remains a scalar string, and formatting matches the native string.
///
/// Clip construction requires the clip-name role explicitly:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::Animation;
/// Animation::new(String::from("walk"));
/// ```
///
/// Direct field construction requires the same role:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::Animation;
/// let _ = Animation { name: String::from("walk"), ..Default::default() };
/// ```
///
/// A clip name cannot be passed directly as a joint query:
///
/// ```compile_fail
/// use fabelgeist_animation::animation::{Animation, AnimationClipName};
/// let clip = Animation::new(AnimationClipName::from("walk"));
/// clip.track(&AnimationClipName::from("hips"));
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
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

impl std::fmt::Debug for AnimationClipName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}
