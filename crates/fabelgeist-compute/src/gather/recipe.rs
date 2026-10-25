use fabelgeist_gpu::data::gpu::ResourceDescriptor;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct GatherRecipe {
    pub(super) input: ResourceDescriptor,
    pub(super) output: ResourceDescriptor,
}
