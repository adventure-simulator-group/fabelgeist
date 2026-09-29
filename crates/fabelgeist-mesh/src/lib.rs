//! Mesh geometry and GPU storage, independent of scene instances and rendering.
mod data;
mod front_face;
mod gpu;
pub mod neighborhood;
mod topology;
pub mod wireframe;
pub use data::MeshData;
pub use front_face::FrontFace;
pub use gpu::GpuMesh;
pub use neighborhood::MeshNeighborhood;
pub use topology::PrimitiveTopology;
pub use wireframe::MeshWireframe;
