//! The groomer's WGSL, as source strings composed the way the compute kernels
//! compose theirs: each kernel's bindings, then the shared helpers it uses.

pub mod common;
pub mod groom;
pub mod guides;
pub mod lod;
pub mod scalp;
pub mod sim;
