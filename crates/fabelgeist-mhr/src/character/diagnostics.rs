//! Nominal quantities and object provenance retained by rig admission failures.
use fabelgeist_fbx::{FbxClassName, FbxObjectId, FbxObjectName, FbxRecordName, Object};

/// A snapshot of the FBX object whose rig interpretation failed.
/// Diagnostic spelling is never used to choose the error classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigObjectContext {
    id: FbxObjectId,
    name: FbxObjectName,
    kind: FbxRecordName,
    class: FbxClassName,
}
impl From<&Object> for RigObjectContext {
    fn from(object: &Object) -> Self {
        Self {
            id: object.id,
            name: object.name.clone(),
            kind: object.kind.clone(),
            class: object.class.clone(),
        }
    }
}
impl std::fmt::Display for RigObjectContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

/// Number of scalar words in one decoded rig property array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RigArrayCount(usize);
impl From<usize> for RigArrayCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
impl std::fmt::Display for RigArrayCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Corners accumulated in a polygon, distinct from mesh vertices and words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolygonCornerCount(usize);
impl From<usize> for PolygonCornerCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
impl std::fmt::Display for PolygonCornerCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The native vertex slot observed by the existing rig decoder.
/// This does not prove that the slot lies within a particular mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RigMeshVertexOrdinal(usize);
impl From<usize> for RigMeshVertexOrdinal {
    fn from(ordinal: usize) -> Self {
        Self(ordinal)
    }
}
impl std::fmt::Display for RigMeshVertexOrdinal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Dense shapes carried by a rig, distinct from model parameter columns.
///
/// ```compile_fail
/// use fabelgeist_mhr::{RigBlendShapeCount, BlendShapeParameterCount};
/// let shapes: RigBlendShapeCount = BlendShapeParameterCount::from(117);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RigBlendShapeCount(usize);
impl From<usize> for RigBlendShapeCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
impl std::fmt::Display for RigBlendShapeCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
