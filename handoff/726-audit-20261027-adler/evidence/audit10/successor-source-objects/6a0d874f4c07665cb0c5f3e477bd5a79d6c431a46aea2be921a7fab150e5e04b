//! Mesh geometry and GPU storage, independent of scene instances and rendering.
//!
//! The GPU half (`GpuMesh`, upload, readback, neighborhood and wireframe
//! buffers) is the `gpu` feature, on by default.
mod data;
mod front_face;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "gpu")]
pub mod neighborhood;
mod topology;
#[cfg(feature = "gpu")]
pub mod wireframe;
pub use data::MeshData;
pub use front_face::FrontFace;
#[cfg(feature = "gpu")]
pub use gpu::GpuMesh;
#[cfg(feature = "gpu")]
pub use neighborhood::MeshNeighborhood;
pub use topology::PrimitiveTopology;
#[cfg(feature = "gpu")]
pub use wireframe::MeshWireframe;
