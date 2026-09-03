//! Running a shader over GPU data.
//!
//! The pipeline, the pass, the resource vocabulary the two speak in, and the
//! binary op that `Texture2d::add` and `Texture2d::mix` are made of. What is
//! *not* here is the algorithm library built on top -- `Map`, `Reduce`,
//! `Scan`, `Sort`, marching cubes and the rest live in `fabelgeist-compute`, which
//! depends on this crate.

mod pass;
mod pipeline;
pub mod signature;
pub mod texture_ops;

pub use pass::*;
pub use pipeline::*;
pub use signature::*;
pub use texture_ops::{TextureBinaryOp, TextureBinaryOpDefinition};

/// Thread-safe cache shared by compute definitions that specialize pipelines by resource shape.
pub type ComputePipelineCache<Key, Value = pipeline::ComputePipeline> =
    std::sync::Arc<std::sync::RwLock<std::collections::HashMap<Key, std::sync::Arc<Value>>>>;

/// Deterministically ordered named resources used as part of a pipeline cache key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderedResourceDescriptors(pub Vec<(String, ResourceDescriptor)>);

impl From<&std::collections::HashMap<String, ResourceDescriptor>> for OrderedResourceDescriptors {
    fn from(resources: &std::collections::HashMap<String, ResourceDescriptor>) -> Self {
        let mut ordered = resources
            .iter()
            .map(|(name, descriptor)| (name.clone(), descriptor.clone()))
            .collect::<Vec<_>>();
        ordered.sort_by(|left, right| left.0.cmp(&right.0));
        Self(ordered)
    }
}

pub mod resource_descriptor;
pub use resource_descriptor::ResourceDescriptor;

pub fn build_compute_pipeline(
    context: &crate::globals::WgpuContext,
    shader: &pipeline::ComputeShader,
    _entry_point: &str,
) -> anyhow::Result<pipeline::ComputePipeline> {
    pipeline::ComputePipeline::new(context, shader.clone())
}
