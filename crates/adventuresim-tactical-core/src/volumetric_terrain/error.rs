//! Bounded terrain surface and landform contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TerrainRecipeError {
    #[error("terrain mesh has invalid cardinality or triangle indices")]
    MeshTopology,
    #[error("erosional patch voxel grid exceeds its bound")]
    ErosionVoxelGridBound,
    #[error("fault patch sample count overflow")]
    SampleCountOverflow,
    #[error("fault patch field is not finite")]
    NonFiniteField,
    #[error("fault patch extraction produced no surface")]
    EmptySurface,
    #[error("fault patch has too many vertices")]
    VertexCountOverflow,

    #[error("terrain surface parameters are outside physical bounds")]
    SurfaceParameters,
    #[error("bedded terrain structure is outside bounds")]
    Bedding,
    #[error("foliated terrain structure is outside bounds")]
    Foliation,
    #[error("terrain geological structure normal is not normalized")]
    StructureNormal,
    #[error("landform tangent is not normalized")]
    Tangent,
    #[error("landform dimensions are outside their bounds")]
    Dimensions,
    #[error("landform does not overlap the playable terrain")]
    OutsidePlayable,
    #[error("fault patch voxel grid exceeds its bound")]
    VoxelGridBound,
}

/// Terrain recipe validation and extraction retain the preventing stage.
pub type TerrainRecipeResult<T> = Result<T, TerrainRecipeError>;
