//! Production child markers, shadow policy and forced visibility ownership.
use super::*;
use crate::presentation::{
    PlayableTreeAggregateWood, PlayableTreeBuds, PlayableTreeCanopyCard,
    PlayableTreeDetailedLeaves, PlayableTreeDetailedTrunk, PlayableTreeDetailedWood,
    PlayableTreeMidTrunk, PlayableTreeTrunk, TreeImpostorProvenance, TreeLeafRepresentation,
    TreeLod, TreeLodCluster, TreeTrunkLod,
};
use bevy::{
    camera::visibility::{NoFrustumCulling, VisibilityRange},
    light::NotShadowCaster,
};

#[derive(SystemParam)]
pub(super) struct RoleInspector<'w, 's> {
    parts: Query<'w, 's, RoleMarkers>,
}

#[derive(bevy::ecs::query::QueryData)]
struct RoleMarkers {
    lod: Option<&'static TreeLod>,
    cluster: Option<&'static TreeLodCluster>,
    leaf: Option<&'static TreeLeafRepresentation>,
    trunk_lod: Has<TreeTrunkLod>,
    trunk: Has<PlayableTreeTrunk>,
    detailed_trunk: Has<PlayableTreeDetailedTrunk>,
    middle_trunk: Has<PlayableTreeMidTrunk>,
    wood: Has<PlayableTreeDetailedWood>,
    buds: Has<PlayableTreeBuds>,
    leaves: Has<PlayableTreeDetailedLeaves>,
    aggregate: Has<PlayableTreeAggregateWood>,
    card: Has<PlayableTreeCanopyCard>,
    no_shadow: Has<NotShadowCaster>,
    no_frustum: Has<NoFrustumCulling>,
    provenance: Option<&'static TreeImpostorProvenance>,
    range: &'static VisibilityRange,
}

impl RoleInspector<'_, '_> {
    pub(super) fn check(
        &self,
        entity: Entity,
        role: InstalledTreePartRole,
        installed: &InstalledTreeSpecimen,
        cache: &TreeCacheInspection,
    ) -> Result<()> {
        let actual = self.parts.get(entity)?;
        ensure!(
            *actual.range == VisibilityRange::abrupt(0.0, f32::MAX),
            "forced child visibility range drift"
        );
        let is_trunk = matches!(
            role,
            InstalledTreePartRole::DetailedTrunk | InstalledTreePartRole::MiddleTrunk
        );
        let is_detailed = role == InstalledTreePartRole::DetailedTrunk;
        let is_middle = role == InstalledTreePartRole::MiddleTrunk;
        let is_wood = matches!(role, InstalledTreePartRole::Wood { .. });
        let is_buds = matches!(role, InstalledTreePartRole::Buds { .. });
        let is_leaves = matches!(
            role,
            InstalledTreePartRole::CamberedLeaves { .. } | InstalledTreePartRole::FlatLeaves { .. }
        );
        let is_aggregate = matches!(role, InstalledTreePartRole::AggregateWood { .. });
        let is_card = matches!(role, InstalledTreePartRole::CanopyCard { .. });
        ensure!(
            actual.trunk_lod == is_trunk
                && actual.trunk == is_trunk
                && actual.detailed_trunk == is_detailed
                && actual.middle_trunk == is_middle
                && actual.wood == is_wood
                && actual.buds == is_buds
                && actual.leaves == is_leaves
                && actual.aggregate == is_aggregate
                && actual.card == is_card,
            "production child role marker drift"
        );
        self.check_layout(&actual, role, installed, cache)?;
        let expected_leaf = match role {
            InstalledTreePartRole::CamberedLeaves { .. } => {
                Some(TreeLeafRepresentation::TexturedMesh)
            }
            InstalledTreePartRole::FlatLeaves { .. } => Some(TreeLeafRepresentation::AlphaCard),
            _ => None,
        };
        ensure!(
            actual.leaf.copied() == expected_leaf,
            "production child leaf representation drift"
        );
        let lod = role.render_lod();
        ensure!(
            actual.lod.map(|value| value.0) == lod,
            "production child LOD marker drift"
        );
        let no_shadow = is_card || (is_aggregate && lod == Some(2));
        let no_frustum = role == InstalledTreePartRole::CanopyCard { lod: 4 };
        ensure!(
            actual.no_shadow == no_shadow && actual.no_frustum == no_frustum,
            "production child shadow/culling policy drift"
        );
        ensure!(
            actual.provenance.is_some() == no_frustum,
            "production child card provenance missing or extra"
        );
        if let Some(provenance) = actual.provenance {
            cache.verify_card(installed, role, provenance)?;
        }
        Ok(())
    }

    fn check_layout(
        &self,
        actual: &RoleMarkersItem,
        role: InstalledTreePartRole,
        installed: &InstalledTreeSpecimen,
        cache: &TreeCacheInspection,
    ) -> Result<()> {
        let primary_group = match role {
            InstalledTreePartRole::Wood { primary_group }
            | InstalledTreePartRole::Buds { primary_group }
            | InstalledTreePartRole::CamberedLeaves { primary_group }
            | InstalledTreePartRole::FlatLeaves { primary_group } => Some(primary_group),
            _ => None,
        };
        match (primary_group, actual.cluster) {
            (Some(expected), Some(actual)) if actual.primary_group == expected => {
                cache.verify_cluster(installed, actual)?
            }
            (None, None) => (),
            _ => anyhow::bail!("production child crown cluster marker drift"),
        }
        Ok(())
    }
}
