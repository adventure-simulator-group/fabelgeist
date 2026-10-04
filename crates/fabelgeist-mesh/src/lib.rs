//! Mesh geometry and GPU storage, independent of scene instances and rendering.
mod data;
mod draw;
mod error;
mod front_face;
mod gpu;
pub mod neighborhood;
mod topology;
pub mod wireframe;
pub use data::MeshData;
pub use draw::{DrawVertexCount, DrawVertexIndex, DrawVertexMembership, MeshAttributeLength};
pub use error::{
    DerivedMeshBuffer, MeshAttribute, MeshDisplacementError, MeshReadbackError,
    MeshTopologyTransferError, MeshTriangleError, MeshUploadError, MeshValidationError,
    SkinWeightViolation,
};
pub use front_face::FrontFace;
pub use gpu::{
    BoxDimensions, DisplacementProjection, DisplacementScale, GpuMesh, MeshDisplacement,
    PlaneGeometry, PlaneSubdivisions, SphereGeometry, SphereRadius, SphereRings, SphereSectors,
};
pub use neighborhood::{MeshNeighborhood, NeighborCapacity};
pub use topology::PrimitiveTopology;
pub use wireframe::MeshWireframe;
