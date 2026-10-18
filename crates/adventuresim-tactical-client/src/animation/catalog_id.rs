//! Authored motion and asset identity admitted by the animation catalog.

use adventuresim_tactical_core::animation::AnimationPackId;
use derive_more::{AsRef, Debug, Display, From};
use std::borrow::Cow;

#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, From)]
#[debug("{_0:?}")]
#[from(String)]
pub(crate) struct MotionId(Cow<'static, str>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MotionKey {
    pub(super) pack: AnimationPackId,
    pub(super) motion: MotionId,
}

#[derive(Clone, Debug, Display, PartialEq, Eq, From, AsRef)]
#[debug("{_0:?}")]
#[from(String)]
#[as_ref(str)]
pub(super) struct AnimationAssetPath(Cow<'static, str>);

impl MotionId {
    pub(super) const fn from_static(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }
    pub(super) fn mirrored(&self) -> Self {
        Self::from(format!("{self}_mirrored"))
    }
}

impl From<&str> for MotionId {
    fn from(name: &str) -> Self {
        Self(Cow::Owned(name.to_owned()))
    }
}

impl MotionKey {
    pub(super) fn new(pack: &AnimationPackId, motion: &MotionId) -> Self {
        Self {
            pack: pack.clone(),
            motion: motion.clone(),
        }
    }
}

impl AnimationAssetPath {
    pub(super) const fn from_static(path: &'static str) -> Self {
        Self(Cow::Borrowed(path))
    }
}
