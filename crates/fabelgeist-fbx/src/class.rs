//! Decoded FBX object subclass roles and their exact format spelling.

use std::fmt;

/// An unrecognized label admitted by its canonical name owner.
/// Its private spelling can be obtained only after known labels are classified.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnknownFbxClassName(String);

/// A decoded object subclass, preserving unknown spelling and suffixes.
/// Known labels have dedicated variants; they do not establish rig membership.
///
/// ```compile_fail
/// use fabelgeist_fbx::{FbxClassName, FbxRecordName, Scene};
/// fn select(scene: &Scene) {
///     scene.objects_of_kind(&FbxClassName::Mesh, &FbxRecordName::Geometry);
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FbxClassName {
    LimbNode,
    Root,
    Null,
    Mesh,
    BlendShape,
    BlendShapeChannel,
    Shape,
    Skin,
    Cluster,
    Unknown(UnknownFbxClassName),
}

impl FbxClassName {
    // Native spelling is used only by formatting.
    fn spelling_for_format(&self) -> &str {
        match self {
            Self::LimbNode => "LimbNode",
            Self::Root => "Root",
            Self::Null => "Null",
            Self::Mesh => "Mesh",
            Self::BlendShape => "BlendShape",
            Self::BlendShapeChannel => "BlendShapeChannel",
            Self::Shape => "Shape",
            Self::Skin => "Skin",
            Self::Cluster => "Cluster",
            Self::Unknown(label) => label.0.as_str(),
        }
    }
}

impl Default for FbxClassName {
    fn default() -> Self {
        Self::from(String::new())
    }
}

impl From<&[u8]> for FbxClassName {
    fn from(encoded: &[u8]) -> Self {
        Self::from(String::from_utf8_lossy(encoded).into_owned())
    }
}

impl From<&str> for FbxClassName {
    fn from(decoded: &str) -> Self {
        Self::from(decoded.to_owned())
    }
}

impl From<String> for FbxClassName {
    fn from(decoded: String) -> Self {
        match decoded.as_str() {
            "LimbNode" => Self::LimbNode,
            "Root" => Self::Root,
            "Null" => Self::Null,
            "Mesh" => Self::Mesh,
            "BlendShape" => Self::BlendShape,
            "BlendShapeChannel" => Self::BlendShapeChannel,
            "Shape" => Self::Shape,
            "Skin" => Self::Skin,
            "Cluster" => Self::Cluster,
            _ => Self::Unknown(UnknownFbxClassName(decoded)),
        }
    }
}

impl fmt::Display for FbxClassName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spelling_for_format())
    }
}

impl fmt::Debug for FbxClassName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.spelling_for_format(), formatter)
    }
}
