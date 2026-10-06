//! Exact byte keys in FBX property tables.
use crate::Prop;

/// An exact property-table key. Unlike record and class names, it is not
/// normalized or lossily decoded before lookup.
///
/// A key borrows its original bytes, including undecodable bytes. It does not
/// classify connection targets or establish the type of a property value.
///
/// ```compile_fail
/// use fabelgeist_fbx::Node;
/// fn select(node: &Node) { let _ = node.property70("RotationOrder"); }
/// ```
///
/// ```compile_fail
/// use fabelgeist_fbx::Node;
/// fn select(node: &Node) { let _ = node.property70_vec3("Lcl Translation", [0.0; 3]); }
/// ```
///
/// ```compile_fail
/// use fabelgeist_fbx::Node;
/// fn select(node: &Node) { let _ = node.property70_i64("RotationOrder", 0); }
/// ```
///
/// ```no_run
/// use fabelgeist_fbx::{FbxPropertyName, Node};
/// fn select(node: &Node) {
///     let _ = node.property70(FbxPropertyName::LOCAL_START);
///     let _ = node.property70_vec3(FbxPropertyName::LOCAL_TRANSLATION, [0.0; 3]);
///     let _ = node.property70_i64(FbxPropertyName::ROTATION_ORDER, 0);
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FbxPropertyName<'a>(&'a [u8]);

impl FbxPropertyName<'static> {
    pub const COLLISION_TYPE: Self = Self(b"col_type");
    pub const ROTATION_ORDER: Self = Self(b"RotationOrder");
    pub const LOCAL_ROTATION: Self = Self(b"Lcl Rotation");
    pub const PRE_ROTATION: Self = Self(b"PreRotation");
    pub const LOCAL_TRANSLATION: Self = Self(b"Lcl Translation");
    pub const LOCAL_SCALING: Self = Self(b"Lcl Scaling");
    pub const LOCAL_START: Self = Self(b"LocalStart");
    pub const LOCAL_STOP: Self = Self(b"LocalStop");
    pub const CURVE_X: Self = Self(b"d|X");
    pub const CURVE_Y: Self = Self(b"d|Y");
    pub const CURVE_Z: Self = Self(b"d|Z");
}
impl<'a> From<&'a [u8]> for FbxPropertyName<'a> {
    fn from(encoded: &'a [u8]) -> Self {
        Self(encoded)
    }
}
impl<'a> FbxPropertyName<'a> {
    pub(crate) fn from_property(property: &'a Prop) -> Option<Self> {
        match property {
            Prop::Str(bytes) | Prop::Raw(bytes) => Some(Self::from(bytes.as_slice())),
            _ => None,
        }
    }
}

impl<'a> From<&'a str> for FbxPropertyName<'a> {
    fn from(name: &'a str) -> Self {
        Self::from(name.as_bytes())
    }
}
