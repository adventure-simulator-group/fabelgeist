//! Exact decoded object subclass identity.
use std::borrow::Cow;

/// A decoded object subclass, preserving unknown spelling and suffixes.
/// It does not establish record role or rig membership.
///
/// ```compile_fail
/// use fabelgeist_fbx::{FbxClassName, FbxRecordName, Scene};
/// fn select(scene: &Scene) {
///     scene.objects_of_kind(&FbxClassName::MESH, &FbxRecordName::GEOMETRY);
/// }
/// ```
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct FbxClassName(Cow<'static, str>);
impl FbxClassName {
    pub const LIMB_NODE: Self = Self(Cow::Borrowed("LimbNode"));
    pub const ROOT: Self = Self(Cow::Borrowed("Root"));
    pub const NULL: Self = Self(Cow::Borrowed("Null"));
    pub const MESH: Self = Self(Cow::Borrowed("Mesh"));
    pub const BLEND_SHAPE: Self = Self(Cow::Borrowed("BlendShape"));
    pub const BLEND_SHAPE_CHANNEL: Self = Self(Cow::Borrowed("BlendShapeChannel"));
    pub const SHAPE: Self = Self(Cow::Borrowed("Shape"));
    pub const SKIN: Self = Self(Cow::Borrowed("Skin"));
    pub const CLUSTER: Self = Self(Cow::Borrowed("Cluster"));
}
impl From<&[u8]> for FbxClassName {
    fn from(encoded: &[u8]) -> Self {
        Self(Cow::Owned(String::from_utf8_lossy(encoded).into_owned()))
    }
}
impl From<&str> for FbxClassName {
    fn from(decoded: &str) -> Self {
        Self(Cow::Owned(decoded.to_owned()))
    }
}
impl std::fmt::Display for FbxClassName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for FbxClassName {
    fn from(decoded: String) -> Self {
        Self(Cow::Owned(decoded))
    }
}
impl std::fmt::Debug for FbxClassName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
