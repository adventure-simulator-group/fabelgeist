//! Canonical leaf selection for sparse and grouped rendering.
use super::*;

/// Selects one stable ordinal lane from every source shoot. Sorting the
/// retained leaves makes the resulting mesh byte-for-byte deterministic even
/// if the source generator changes its iteration order.
pub(super) fn sparse_woody_far_card_leaves(leaves: &[TreeLeaf]) -> Vec<TreeLeaf> {
    let mut retained = leaves
        .iter()
        .filter(|leaf| sparse_woody_far_card_retained(**leaf))
        .copied()
        .collect::<Vec<_>>();
    retained.sort_unstable_by_key(|leaf| {
        (
            leaf.primary_group,
            leaf.secondary_group,
            leaf.shoot_id,
            leaf.leaf_ordinal,
        )
    });
    retained
}

pub(super) fn sparse_woody_far_card_retained(leaf: TreeLeaf) -> bool {
    let shoot_key = leaf.shoot_id;
    let retained_ordinal_lane = streams::RETAINED_LANE.rng(shoot_key, &[]).index(4) as u8;
    leaf.leaf_ordinal & 3 == retained_ordinal_lane
}

/// Returns the deterministic 75% subset used exclusively by streamed
/// playable-tree flat cards. Each shoot omits one of its four ordinal lanes;
/// the omitted lane is salted by stable source identity, keeping the crown
/// distributed when source vectors are reordered. Cambered leaves and baked
/// aggregate canopy cards deliberately retain the full source set.
pub(in crate::presentation) fn detailed_flat_card_group_leaves(
    leaves: &[TreeLeaf],
    primary_group: u8,
) -> Vec<TreeLeaf> {
    let mut retained = leaves
        .iter()
        .filter(|leaf| {
            leaf.primary_group == primary_group && detailed_flat_card_leaf_retained(**leaf)
        })
        .copied()
        .collect::<Vec<_>>();
    retained.sort_unstable_by_key(|leaf| {
        (
            leaf.primary_group,
            leaf.secondary_group,
            leaf.shoot_id,
            leaf.leaf_ordinal,
        )
    });
    retained
}

pub(super) fn detailed_flat_card_leaf_retained(leaf: TreeLeaf) -> bool {
    let shoot_key = leaf.shoot_id;
    let omitted_ordinal_lane = streams::OMITTED_LANE.rng(shoot_key, &[]).index(4) as u8;
    leaf.leaf_ordinal & 3 != omitted_ordinal_lane
}
