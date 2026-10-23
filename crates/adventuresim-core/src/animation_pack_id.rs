//! Open authored animation-pack identity shared by item and tactical catalogues.

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnimationPackId(Cow<'static, str>);

impl AnimationPackId {
    pub const HUMANOID_UNARMED: Self = Self(Cow::Borrowed("humanoid_unarmed"));
    pub const HUMANOID_2H_CLOSE: Self = Self(Cow::Borrowed("humanoid_2h_close"));
}

impl fmt::Debug for AnimationPackId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for AnimationPackId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl AsRef<str> for AnimationPackId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for AnimationPackId {
    fn from(id: String) -> Self {
        Self(Cow::Owned(id))
    }
}

impl From<&str> for AnimationPackId {
    fn from(id: &str) -> Self {
        Self(Cow::Owned(id.to_owned()))
    }
}
