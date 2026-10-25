use crate::resource_recipe::SecondaryResources;
use crate::BoundaryMode;
use fabelgeist_gpu::data::gpu::ResourceDescriptor;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct StencilRecipe {
    pub(super) input: ResourceDescriptor,
    pub(super) output: ResourceDescriptor,
    pub(super) secondary: SecondaryResources,
    pub(super) boundary_mode: BoundaryMode,
}
