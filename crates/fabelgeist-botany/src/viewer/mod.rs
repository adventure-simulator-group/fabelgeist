//! The botany viewer's engine: the simulation thread, its snapshots, the
//! forest's levels of detail and the wgpu renderer. The interface over it
//! belongs to the application embedding it.

pub mod camera;
pub mod hlod;
pub mod profile;
pub mod renderer;
pub mod sim;
