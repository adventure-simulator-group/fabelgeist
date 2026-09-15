//! Botanical organs compose into deterministic meshes in metres, rooted at Y=0.
//! Species are authored parameter presets; geometry never branches on species.
pub mod flower;
pub mod habitat;
mod mesh;
mod parameters;
pub use mesh::{PlantMesh, Tessellation};
pub use parameters::{GenerationError, Pigment};
