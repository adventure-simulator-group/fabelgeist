//! Object subclass identity and the structural role of a Model record.
use crate::Prop;
use fabelgeist_storage::StorageView;
use std::borrow::Cow;

/// A decoded object subclass, preserving unknown spelling and suffixes.
///
/// ```compile_fail
/// use fabelgeist_fbx::{FbxClassName, FbxRecordName, Scene};
/// let scene = Scene::from_roots(Vec::new());
/// scene.objects_of_kind(&FbxClassName::MESH, &FbxRecordName::GEOMETRY);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
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

    pub(crate) fn from_property(property: &Prop) -> Option<Self> {
        match property {
            Prop::Str(bytes) | Prop::Raw(bytes) => {
                Some(Self::from(StorageView::from(bytes.as_slice())))
            }
            _ => None,
        }
    }
}
impl From<StorageView<'_>> for FbxClassName {
    fn from(encoded: StorageView<'_>) -> Self {
        Self(Cow::Owned(
            String::from_utf8_lossy(encoded.as_ref()).into_owned(),
        ))
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

/// The structural role of an FBX Model, without asserting rig membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelRole {
    /// Both Root and LimbNode are joint candidates, as in OpenFBX.
    Joint,
    /// A grouping node, locator, or collision primitive interpreted by the rig.
    Null,
    /// A Model with another subclass, including meshes and unknown classes.
    Uninterpreted,
}
