//! Open authored skeleton-family identity shared by packs and rig caches.

use derive_more::{AsRef, Debug, Display, From};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, AsRef, From,
)]
#[serde(transparent)]
#[debug("{_0:?}")]
#[as_ref(str)]
#[from(String)]
pub struct SkeletonFamily(Cow<'static, str>);

impl SkeletonFamily {
    pub const HUMANOID: Self = Self(Cow::Borrowed("humanoid"));
}

impl From<&str> for SkeletonFamily {
    fn from(value: &str) -> Self {
        Self(Cow::Owned(value.to_owned()))
    }
}
