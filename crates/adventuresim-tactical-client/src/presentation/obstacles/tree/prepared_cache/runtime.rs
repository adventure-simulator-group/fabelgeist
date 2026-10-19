//! Move checked wire arrays into the production numerical mesh boundary.
use super::super::{TreeBranchSegment, TreeLeaf};
use super::*;
use std::sync::Arc;

pub(in crate::presentation::obstacles::tree) struct FullVariant {
    pub(in crate::presentation::obstacles::tree) key: Seed,
    pub(in crate::presentation::obstacles::tree) seed: Seed,
    pub(in crate::presentation::obstacles::tree) species: TreePresentationSpecies,
    pub(in crate::presentation::obstacles::tree) content: blake3::Hash,
    pub(in crate::presentation::obstacles::tree) branches: Arc<[TreeBranchSegment]>,
    pub(in crate::presentation::obstacles::tree) leaves: Arc<[TreeLeaf]>,
    pub(in crate::presentation::obstacles::tree) detailed_trunk: Mesh,
    pub(in crate::presentation::obstacles::tree) mid_trunk: Mesh,
    pub(in crate::presentation::obstacles::tree) groups: Vec<FullCluster>,
    pub(in crate::presentation::obstacles::tree) card: TreeLodBake,
}

/// Primary-group scalar and local metres cross the existing tree mesh kernel.
pub(in crate::presentation::obstacles::tree) struct FullCluster {
    pub(in crate::presentation::obstacles::tree) primary_group: u8,
    pub(in crate::presentation::obstacles::tree) center: Vec3,
    pub(in crate::presentation::obstacles::tree) radius: f32,
    pub(in crate::presentation::obstacles::tree) wood: Mesh,
    pub(in crate::presentation::obstacles::tree) buds: Mesh,
    pub(in crate::presentation::obstacles::tree) leaves: Mesh,
}

impl PreparedTreeCache {
    pub(in crate::presentation::obstacles::tree) fn into_variant(
        self,
        environment: &SceneEnvironment,
    ) -> Result<FullVariant> {
        ensure!(
            self.payload.binding.environment == serde_json::to_vec(environment)?,
            "prepared tree destination environment changed"
        );
        ensure!(
            self.payload.binding.cache_key
                == playable_cache_key(
                    self.payload.binding.species,
                    self.payload.binding.seed,
                    environment
                ),
            "prepared tree destination site changed"
        );
        let Payload {
            binding,
            source,
            detailed_trunk,
            mid_trunk,
            groups,
            card,
        } = self.payload;
        let (branches, leaves) = source.runtime();
        Ok(FullVariant {
            key: binding.cache_key,
            seed: binding.seed,
            species: binding.species,
            content: self.digest,
            branches: branches.into(),
            leaves: leaves.into(),
            detailed_trunk: detailed_trunk.runtime(),
            mid_trunk: mid_trunk.runtime(),
            groups: groups
                .into_iter()
                .map(|group| FullCluster {
                    primary_group: group.primary_group,
                    center: Vec3::from_array(group.center),
                    radius: group.radius,
                    wood: group.wood.runtime(),
                    buds: group.buds.runtime(),
                    leaves: group.leaves.runtime(),
                })
                .collect(),
            card: card.runtime(),
        })
    }
}
