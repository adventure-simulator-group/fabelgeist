use super::{PolygonCornerCount, RigArrayCount, RigMeshVertexOrdinal, RigObjectContext};
use crate::MeshVertexCount;
use fabelgeist_fbx::FbxDecodeError;

#[derive(Debug)]
pub enum CharacterDecodeError {
    Fbx(FbxDecodeError),
    NoJoints,
    MissingMesh,
    MissingGeometry(RigObjectContext),
    MissingVertices(RigObjectContext),
    IncompletePosition {
        geometry: RigObjectContext,
        values: RigArrayCount,
    },
    MissingPolygons(RigObjectContext),
    PolygonTooShort(PolygonCornerCount),
    TrailingPolygon(PolygonCornerCount),
    IncompleteTexcoord {
        geometry: RigObjectContext,
        values: RigArrayCount,
    },
    MissingSkin(RigObjectContext),
    UnknownBone(RigObjectContext),
    SkinArrayMismatch {
        cluster: RigObjectContext,
        indices: RigArrayCount,
        weights: RigArrayCount,
    },
    SkinVertex {
        cluster: RigObjectContext,
        vertex: RigMeshVertexOrdinal,
        vertices: MeshVertexCount,
    },
    NoSkinWeights(RigMeshVertexOrdinal),
    EmptySkinWeightSum(RigMeshVertexOrdinal),
}
impl std::fmt::Display for CharacterDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fbx(source) => source.fmt(f),
            Self::NoJoints => f.write_str("no joints found in FBX rig"),
            Self::MissingMesh => f.write_str("FBX rig has no mesh"),
            Self::MissingGeometry(_) => f.write_str("mesh model has no geometry"),
            Self::MissingVertices(_) => f.write_str("mesh geometry has no vertices"),
            Self::IncompletePosition { .. } => {
                f.write_str("FBX vertex array contains an incomplete three-axis position")
            }
            Self::MissingPolygons(_) => f.write_str("mesh geometry has no polygons"),
            Self::PolygonTooShort(corners) => {
                write!(
                    f,
                    "invalid face with {corners} indices; expected at least 3"
                )
            }
            Self::TrailingPolygon(_) => {
                f.write_str("trailing polygon indices without a terminator")
            }
            Self::IncompleteTexcoord { .. } => {
                f.write_str("FBX UV array contains an incomplete coordinate pair")
            }
            Self::MissingSkin(geometry) => {
                write!(f, "geometry '{geometry}' has no skin deformer")
            }
            Self::UnknownBone(cluster) => {
                write!(f, "cluster '{cluster}' references an unknown bone")
            }
            Self::SkinArrayMismatch { cluster, .. } => {
                write!(f, "cluster '{cluster}' has mismatched indices and weights")
            }
            Self::SkinVertex {
                cluster, vertex, ..
            } => {
                write!(f, "cluster '{cluster}' references vertex {vertex}")
            }
            Self::NoSkinWeights(vertex) => write!(f, "no skinning weights for vertex {vertex}"),
            Self::EmptySkinWeightSum(vertex) => write!(f, "empty weight sum for vertex {vertex}"),
        }
    }
}
impl std::error::Error for CharacterDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Fbx(source) => Some(source),
            _ => None,
        }
    }
}
