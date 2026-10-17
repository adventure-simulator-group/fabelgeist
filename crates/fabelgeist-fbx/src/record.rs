//! Decoded FBX record roles and their exact format spelling.

use std::fmt;

/// An unrecognized label admitted by its canonical name owner.
/// Its private spelling can be obtained only after known labels are classified.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnknownFbxRecordName(String);

/// A record name, distinct from an object class or a property-table key.
/// Known labels have dedicated variants; unknown decoded spelling is preserved.
/// Binary admission uses lossy UTF-8 decoding without normalization.
///
/// ```compile_fail
/// use fabelgeist_fbx::Node;
/// fn select(node: &Node) { let _ = node.child("Vertices"); }
/// ```
///
/// ```no_run
/// use fabelgeist_fbx::{FbxClassName, FbxRecordName, Node, Scene};
/// fn select(node: &Node, scene: &Scene) {
///     let _ = node.child(&FbxRecordName::Vertices);
///     let _ = scene.objects_of_kind(&FbxRecordName::Geometry, &FbxClassName::Mesh).next();
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FbxRecordName {
    Objects,
    Connections,
    Model,
    Geometry,
    Deformer,
    AnimationStack,
    AnimationLayer,
    AnimationCurveNode,
    AnimationCurve,
    Properties70,
    KeyTime,
    KeyValueFloat,
    LayerElementUv,
    Uv,
    ReferenceInformationType,
    UvIndex,
    Vertices,
    Indexes,
    TransformLink,
    Weights,
    PolygonVertexIndex,
    Unknown(UnknownFbxRecordName),
}

impl FbxRecordName {
    /// The decoded name encoded for an FBX node record.
    pub fn encoded(&self) -> &[u8] {
        self.spelling_for_format().as_bytes()
    }

    // Native spelling is used only by formatting and wire encoding.
    fn spelling_for_format(&self) -> &str {
        match self {
            Self::Objects => "Objects",
            Self::Connections => "Connections",
            Self::Model => "Model",
            Self::Geometry => "Geometry",
            Self::Deformer => "Deformer",
            Self::AnimationStack => "AnimationStack",
            Self::AnimationLayer => "AnimationLayer",
            Self::AnimationCurveNode => "AnimationCurveNode",
            Self::AnimationCurve => "AnimationCurve",
            Self::Properties70 => "Properties70",
            Self::KeyTime => "KeyTime",
            Self::KeyValueFloat => "KeyValueFloat",
            Self::LayerElementUv => "LayerElementUV",
            Self::Uv => "UV",
            Self::ReferenceInformationType => "ReferenceInformationType",
            Self::UvIndex => "UVIndex",
            Self::Vertices => "Vertices",
            Self::Indexes => "Indexes",
            Self::TransformLink => "TransformLink",
            Self::Weights => "Weights",
            Self::PolygonVertexIndex => "PolygonVertexIndex",
            Self::Unknown(label) => label.0.as_str(),
        }
    }
}

impl Default for FbxRecordName {
    fn default() -> Self {
        Self::from(String::new())
    }
}

impl From<&[u8]> for FbxRecordName {
    fn from(encoded: &[u8]) -> Self {
        Self::from(String::from_utf8_lossy(encoded).into_owned())
    }
}

impl From<&str> for FbxRecordName {
    fn from(decoded: &str) -> Self {
        Self::from(decoded.to_owned())
    }
}

impl From<String> for FbxRecordName {
    fn from(decoded: String) -> Self {
        match decoded.as_str() {
            "Objects" => Self::Objects,
            "Connections" => Self::Connections,
            "Model" => Self::Model,
            "Geometry" => Self::Geometry,
            "Deformer" => Self::Deformer,
            "AnimationStack" => Self::AnimationStack,
            "AnimationLayer" => Self::AnimationLayer,
            "AnimationCurveNode" => Self::AnimationCurveNode,
            "AnimationCurve" => Self::AnimationCurve,
            "Properties70" => Self::Properties70,
            "KeyTime" => Self::KeyTime,
            "KeyValueFloat" => Self::KeyValueFloat,
            "LayerElementUV" => Self::LayerElementUv,
            "UV" => Self::Uv,
            "ReferenceInformationType" => Self::ReferenceInformationType,
            "UVIndex" => Self::UvIndex,
            "Vertices" => Self::Vertices,
            "Indexes" => Self::Indexes,
            "TransformLink" => Self::TransformLink,
            "Weights" => Self::Weights,
            "PolygonVertexIndex" => Self::PolygonVertexIndex,
            _ => Self::Unknown(UnknownFbxRecordName(decoded)),
        }
    }
}

impl fmt::Display for FbxRecordName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.spelling_for_format())
    }
}

impl fmt::Debug for FbxRecordName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.spelling_for_format(), formatter)
    }
}
