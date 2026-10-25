//! Canonical secondary binding identity for generated resource recipes.

use fabelgeist_gpu::data::gpu::ResourceDescriptor;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SecondaryResources(Vec<SecondaryResource>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SecondaryResource {
    name: String,
    descriptor: ResourceDescriptor,
}

impl From<&HashMap<String, ResourceDescriptor>> for SecondaryResources {
    fn from(resources: &HashMap<String, ResourceDescriptor>) -> Self {
        let mut bindings: Vec<_> = resources
            .iter()
            .map(|(name, descriptor)| SecondaryResource {
                name: name.clone(),
                descriptor: descriptor.clone(),
            })
            .collect();
        bindings.sort_by(|left, right| left.name.cmp(&right.name));
        Self(bindings)
    }
}
