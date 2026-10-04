//! Mesh admission and transfer failures with retained roles and causes.

use fabelgeist_gpu::prelude::{
    Buffer, BufferCreationError, BufferDefinition, BufferUpload, ComputePassError,
    ComputePipelineError, ReadbackError, WgpuContext,
};

use crate::{DrawVertexCount, DrawVertexIndex, MeshAttributeLength, PrimitiveTopology};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshAttribute {
    Positions,
    Normals,
    TextureCoordinates,
    Indices,
    Joints,
    Weights,
}

impl std::fmt::Display for MeshAttribute {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Positions => "position",
            Self::Normals => "normal",
            Self::TextureCoordinates => "UV",
            Self::Indices => "index",
            Self::Joints => "joint",
            Self::Weights => "weight",
        })
    }
}

impl MeshAttribute {
    pub(crate) fn allocate(
        self,
        context: &WgpuContext,
        payload: BufferUpload<'_>,
        definition: BufferDefinition,
    ) -> Result<Buffer, MeshUploadError> {
        Buffer::from_upload(context, payload, definition).map_err(
            |source: BufferCreationError| -> MeshUploadError {
                MeshUploadError::allocation(self, source)
            },
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkinWeightViolation {
    Length {
        expected: MeshAttributeLength,
        actual: MeshAttributeLength,
    },
    NonFinite,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MeshValidationError {
    #[error("too many mesh vertices")]
    TooManyVertices { actual: MeshAttributeLength },
    #[error("non-finite mesh position")]
    NonFinitePosition,
    #[error("{attribute} count must match positions")]
    AttributeLength {
        attribute: MeshAttribute,
        expected: MeshAttributeLength,
        actual: MeshAttributeLength,
    },
    #[error("non-finite mesh attribute")]
    NonFiniteAttribute { attribute: MeshAttribute },
    #[error("joints and weights must be supplied together")]
    SkinPair { missing: MeshAttribute },
    #[error("invalid skin weights")]
    SkinWeights { violation: SkinWeightViolation },
    #[error("mesh index out of bounds")]
    IndexOutOfBounds {
        index: DrawVertexIndex,
        vertices: DrawVertexCount,
    },
    #[error("triangle list needs triples")]
    TriangleListCount { actual: MeshAttributeLength },
    #[error("line list needs pairs")]
    LineListCount { actual: MeshAttributeLength },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MeshTriangleError {
    #[error("{0}")]
    Validation(#[from] MeshValidationError),
    #[error("operation requires triangle geometry")]
    UnsupportedTopology { topology: PrimitiveTopology },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MeshUploadError {
    #[error("{0}")]
    Validation(#[from] MeshValidationError),
    #[error("{source}")]
    Allocation {
        attribute: MeshAttribute,
        #[source]
        source: BufferCreationError,
    },
}

impl MeshUploadError {
    pub(crate) fn allocation(attribute: MeshAttribute, source: BufferCreationError) -> Self {
        Self::Allocation { attribute, source }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub struct MeshReadbackError {
    pub attribute: MeshAttribute,
    #[source]
    pub source: ReadbackError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerivedMeshBuffer {
    Neighborhood,
    Wireframe,
    NeighborLines,
}

impl DerivedMeshBuffer {
    pub(crate) fn upload(
        self,
        context: &WgpuContext,
        payload: BufferUpload<'_>,
    ) -> Result<Buffer, MeshTopologyTransferError> {
        let definition = match self {
            Self::Neighborhood => BufferDefinition::storage().with_label("Vertex Neighbors".into()),
            Self::Wireframe => {
                BufferDefinition::index().with_label("Wireframe Line Indices".into())
            }
            Self::NeighborLines => {
                BufferDefinition::index().with_label("Neighbor Lines Indices".into())
            }
        };
        Buffer::from_upload(context, payload, definition).map_err(
            |source: BufferCreationError| -> MeshTopologyTransferError {
                MeshTopologyTransferError::Allocation {
                    output: self,
                    source,
                }
            },
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MeshTopologyTransferError {
    #[error("{0}")]
    ReadIndices(#[source] ReadbackError),
    #[error("{0}")]
    ReadNeighbors(#[source] ReadbackError),
    #[error("{source}")]
    Allocation {
        output: DerivedMeshBuffer,
        #[source]
        source: BufferCreationError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum MeshDisplacementError {
    #[error("{0}")]
    Pipeline(#[from] ComputePipelineError),
    #[error("{0}")]
    Positions(#[from] BufferCreationError),
    #[error("{0}")]
    Dispatch(#[from] ComputePassError),
}
