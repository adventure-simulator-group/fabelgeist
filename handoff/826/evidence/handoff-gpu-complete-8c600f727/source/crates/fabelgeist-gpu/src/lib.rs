//! The GPU layer.
//!
//! What you need to hold data on a GPU and run a shader over it: the
//! [`WgpuContext`](globals::WgpuContext), buffers, textures, samplers,
//! shaders, bind groups, and the compute pipeline and pass that bind them
//! together. Nothing here knows what is being drawn or solved.
//!
//! What is built on it lives elsewhere: the algorithm library in
//! `fabelgeist-compute`, rendering and cameras in `prism-render`, UI styling in
//! `prism-ui`.

pub mod data;
pub mod globals;
pub mod prelude;
