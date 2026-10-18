//! Object identity as encoded by the FBX object and connection tables.
use crate::Prop;

/// An identity in one FBX scene, distinct from array indices and time ticks.
///
/// Every signed value is representable. Identity alone proves neither object
/// existence nor uniqueness: duplicate records retain their file order, and
/// scene lookup resolves the last record with the requested identity.
///
/// ```compile_fail
/// use fabelgeist_fbx::Scene;
/// let scene = Scene::from_roots(Vec::new());
/// scene.get(42);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FbxObjectId(i64);

impl FbxObjectId {
    /// Connection destination for objects attached to the scene root.
    /// Connections whose source has this identity are ignored by the reader.
    pub const SCENE_ROOT: Self = Self(0);

    /// Admit the integer property encodings accepted by the object table.
    /// Boolean properties retain the reader's existing zero/one conversion.
    pub(crate) fn from_property(property: &Prop) -> Option<Self> {
        match property {
            Prop::I16(value) => Some(Self(i64::from(*value))),
            Prop::I32(value) => Some(Self(i64::from(*value))),
            Prop::I64(value) => Some(Self(*value)),
            Prop::Bool(value) => Some(Self(i64::from(*value))),
            _ => None,
        }
    }
}

impl From<i64> for FbxObjectId {
    fn from(encoded: i64) -> Self {
        Self(encoded)
    }
}

impl From<FbxObjectId> for Prop {
    fn from(identity: FbxObjectId) -> Self {
        Self::I64(identity.0)
    }
}
