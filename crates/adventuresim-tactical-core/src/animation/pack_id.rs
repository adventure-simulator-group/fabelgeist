//! Exact authored animation-pack identity, independent of clip motion names.

use derive_more::{AsRef, Debug, Display, From};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, AsRef, Display, From,
)]
#[serde(transparent)]
#[debug("{_0:?}")]
#[as_ref(str)]
#[from(String)]
pub struct AnimationPackId(Cow<'static, str>);

impl AnimationPackId {
    pub const HUMANOID_UNARMED: Self = Self(Cow::Borrowed("humanoid_unarmed"));
    pub const HUMANOID_2H_CLOSE: Self = Self(Cow::Borrowed("humanoid_2h_close"));
}

impl From<&str> for AnimationPackId {
    fn from(id: &str) -> Self {
        Self(Cow::Owned(id.to_owned()))
    }
}
