//! Exact decoded FBX record identity and its serialized UTF-8 representation.
use std::borrow::Cow;

/// A record name, distinct from an object class or a property-table key.
/// Unknown names retain their decoded spelling without normalization.
/// Binary admission uses lossy UTF-8 decoding, as the record reader does.
///
/// ```compile_fail
/// use fabelgeist_fbx::Node;
/// fn select(node: &Node) { let _ = node.child("Vertices"); }
/// ```
///
/// ```no_run
/// use fabelgeist_fbx::{FbxClassName, FbxRecordName, Node, Scene};
/// fn select(node: &Node, scene: &Scene) {
///     let _ = node.child(&FbxRecordName::VERTICES);
///     let _ = scene.objects_of_kind(&FbxRecordName::GEOMETRY, &FbxClassName::MESH).next();
/// }
/// ```
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct FbxRecordName(Cow<'static, str>);

impl FbxRecordName {
    pub const OBJECTS: Self = Self(Cow::Borrowed("Objects"));
    pub const CONNECTIONS: Self = Self(Cow::Borrowed("Connections"));
    pub const MODEL: Self = Self(Cow::Borrowed("Model"));
    pub const GEOMETRY: Self = Self(Cow::Borrowed("Geometry"));
    pub const DEFORMER: Self = Self(Cow::Borrowed("Deformer"));
    pub const ANIMATION_STACK: Self = Self(Cow::Borrowed("AnimationStack"));
    pub const ANIMATION_LAYER: Self = Self(Cow::Borrowed("AnimationLayer"));
    pub const ANIMATION_CURVE_NODE: Self = Self(Cow::Borrowed("AnimationCurveNode"));
    pub const ANIMATION_CURVE: Self = Self(Cow::Borrowed("AnimationCurve"));
    pub const PROPERTIES70: Self = Self(Cow::Borrowed("Properties70"));
    pub const KEY_TIME: Self = Self(Cow::Borrowed("KeyTime"));
    pub const KEY_VALUE_FLOAT: Self = Self(Cow::Borrowed("KeyValueFloat"));
    pub const LAYER_ELEMENT_UV: Self = Self(Cow::Borrowed("LayerElementUV"));
    pub const UV: Self = Self(Cow::Borrowed("UV"));
    pub const REFERENCE_INFORMATION_TYPE: Self = Self(Cow::Borrowed("ReferenceInformationType"));
    pub const UV_INDEX: Self = Self(Cow::Borrowed("UVIndex"));
    pub const VERTICES: Self = Self(Cow::Borrowed("Vertices"));
    pub const INDEXES: Self = Self(Cow::Borrowed("Indexes"));
    pub const TRANSFORM_LINK: Self = Self(Cow::Borrowed("TransformLink"));
    pub const WEIGHTS: Self = Self(Cow::Borrowed("Weights"));
    pub const POLYGON_VERTEX_INDEX: Self = Self(Cow::Borrowed("PolygonVertexIndex"));

    /// The decoded name encoded for an FBX node record.
    pub fn encoded(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
impl From<&[u8]> for FbxRecordName {
    fn from(encoded: &[u8]) -> Self {
        Self(Cow::Owned(String::from_utf8_lossy(encoded).into_owned()))
    }
}
impl From<&str> for FbxRecordName {
    fn from(decoded: &str) -> Self {
        Self(Cow::Owned(decoded.to_owned()))
    }
}
impl std::fmt::Display for FbxRecordName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for FbxRecordName {
    fn from(decoded: String) -> Self {
        Self(Cow::Owned(decoded))
    }
}
impl std::fmt::Debug for FbxRecordName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
