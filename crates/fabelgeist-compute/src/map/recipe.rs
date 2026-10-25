use crate::resource_recipe::SecondaryResources;
use fabelgeist_gpu::data::gpu::ResourceDescriptor;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct MapRecipe {
    pub(super) input: Option<ResourceDescriptor>,
    pub(super) output: ResourceDescriptor,
    pub(super) secondary: SecondaryResources,
}
