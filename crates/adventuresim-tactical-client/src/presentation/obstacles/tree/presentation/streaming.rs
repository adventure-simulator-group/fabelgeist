//! Which representation tiers a playable tree streams in, and which of its
//! resident children stay enabled.
//!
//! A tier is one bit of the residency masks: bit 0 holds both trunks, bit 1
//! the detailed crown (buds, wood and leaves), bits 2 to 4 aggregate LODs 1 to
//! 3 and bit 5 the whole-tree card. Leaves also carry a leaf bit: 1 for
//! textured meshes, 2 for alpha cards. Residency only grows, because
//! respawning a returning tier stalled frames; tiers the camera has left are
//! disabled instead, so they drop out of every per-frame query.

use std::ops::Range;

use bevy::{ecs::entity_disabling::Disabled, prelude::*};

use super::super::impostor::{TREE_LEAF_HANDOFF_END, TREE_LEAF_HANDOFF_START};
use super::super::{TreeLeafRepresentation, TreeLodRenderOverride};
use crate::presentation::range_dormancy::wake;

/// Projected-scale distances over which each tier is requested. Each incoming
/// crown tier is generated one full outgoing LOD band before it becomes
/// visible: mesh and material insertion is asynchronous in the render world,
/// and a shorter lead left only the already-resident trunk on a first
/// approach. This changes when residency is paid, not which tiers are drawn.
const TIER_REQUEST_RANGES: [Range<f32>; 6] = [
    0.0..75.0,
    0.0..24.0,
    3.0..44.0,
    8.0..75.0,
    18.0..100.0,
    40.0..220.0,
];

/// How far, as a ratio of projected-scale distance, a tier's request range is
/// widened before its children go dormant. Crown clusters scale their
/// visibility ranges by up to 1/0.65 of the base bands, so the ratio exceeds
/// that: no dormant child is ever inside its own visibility range.
const DORMANCY_DISTANCE_RATIO: f32 = 1.6;

const TEXTURED_LEAF_BIT: u8 = 1;
const ALPHA_CARD_LEAF_BIT: u8 = 2;

pub(super) type StreamedTreeChildState<'a> = (
    Entity,
    &'a StreamedTreeChild,
    &'a ViewVisibility,
    Has<Disabled>,
);

/// The tiers and leaf representations of one playable tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::presentation) struct TreeTiers {
    pub(super) mask: u8,
    pub(super) leaf: u8,
}

/// The tier a streamed child of a playable tree belongs to.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::presentation) struct StreamedTreeChild {
    tier: u8,
    leaf: Option<u8>,
}

impl TreeTiers {
    pub(super) const ALL: Self = Self {
        mask: u8::MAX,
        leaf: u8::MAX,
    };

    /// Tiers whose children stay enabled at `scaled_distance`: every tier
    /// requested within [`DORMANCY_DISTANCE_RATIO`] of it, plus whatever is
    /// requested now. A forced LOD keeps every child awake; the override
    /// hides unselected tiers itself.
    pub(super) fn retained(
        scaled_distance: f32,
        lod_override: &TreeLodRenderOverride,
        requested: Self,
    ) -> Self {
        if lod_override.lod.is_some() {
            return Self::ALL;
        }
        let near = scaled_distance / DORMANCY_DISTANCE_RATIO;
        let far = scaled_distance * DORMANCY_DISTANCE_RATIO;
        let mask = TIER_REQUEST_RANGES
            .iter()
            .enumerate()
            .filter(|(_, range)| range.start <= far && near < range.end)
            .fold(0, |mask, (bit, _)| mask | (1 << bit));
        let textured = if near <= TREE_LEAF_HANDOFF_END {
            TEXTURED_LEAF_BIT
        } else {
            0
        };
        let cards = if far >= TREE_LEAF_HANDOFF_START {
            ALPHA_CARD_LEAF_BIT
        } else {
            0
        };
        let leaf = textured | cards;
        Self {
            mask: mask | requested.mask,
            leaf: leaf | requested.leaf,
        }
    }
}

impl StreamedTreeChild {
    pub(super) const TRUNK: Self = Self::tier(0);
    pub(super) const DETAILED_CROWN: Self = Self::tier(1);
    pub(super) const TEXTURED_LEAVES: Self = Self {
        tier: 1 << 1,
        leaf: Some(TEXTURED_LEAF_BIT),
    };
    pub(super) const LEAF_CARDS: Self = Self {
        tier: 1 << 1,
        leaf: Some(ALPHA_CARD_LEAF_BIT),
    };

    /// Aggregate LODs 1 to 3 and the whole-tree card at LOD 4.
    pub(super) const fn aggregate(lod: u8) -> Self {
        Self::tier(lod + 1)
    }

    const fn tier(bit: u8) -> Self {
        Self {
            tier: 1 << bit,
            leaf: None,
        }
    }

    fn is_retained(self, retained: TreeTiers) -> bool {
        retained.mask & self.tier != 0 && self.leaf.is_none_or(|leaf| retained.leaf & leaf != 0)
    }

    fn tiers(self) -> TreeTiers {
        TreeTiers {
            mask: self.tier,
            leaf: self.leaf.unwrap_or(0),
        }
    }
}

/// The tiers to make resident at `scaled_distance`.
pub(super) fn tree_streaming_mask(scaled_distance: f32) -> u8 {
    TIER_REQUEST_RANGES
        .iter()
        .enumerate()
        .filter(|(_, range)| range.contains(&scaled_distance))
        .fold(0, |mask, (bit, _)| mask | (1 << bit))
}

pub(super) fn tree_streamed_leaf_representation(
    scaled_distance: f32,
) -> Option<TreeLeafRepresentation> {
    // Keep residency decisions on the same projected-scale-aware base band as
    // tree_leaf_visibility. The cluster-specific visibility range may widen
    // this overlap, but it must never request a representation after its
    // corresponding visibility range has become active.
    if scaled_distance < TREE_LEAF_HANDOFF_START {
        Some(TreeLeafRepresentation::TexturedMesh)
    } else if scaled_distance > TREE_LEAF_HANDOFF_END {
        Some(TreeLeafRepresentation::AlphaCard)
    } else {
        None
    }
}

/// The leaf representations to make resident with the detailed crown.
pub(super) fn tree_leaf_mask(mask: u8, selected: Option<TreeLeafRepresentation>) -> u8 {
    if mask & StreamedTreeChild::DETAILED_CROWN.tier == 0 {
        return 0;
    }
    match selected {
        Some(TreeLeafRepresentation::TexturedMesh) => TEXTURED_LEAF_BIT,
        Some(TreeLeafRepresentation::AlphaCard) => ALPHA_CARD_LEAF_BIT,
        None => TEXTURED_LEAF_BIT | ALPHA_CARD_LEAF_BIT,
    }
}

/// Disables children outside `retained` and wakes those inside it. Returns
/// the tiers actually applied: a child drawn last frame is left awake until it
/// is culled, and its tier is kept in the result so the caller retries.
pub(super) fn apply_tree_child_dormancy(
    commands: &mut Commands,
    children: &Children,
    states: &Query<StreamedTreeChildState, Allow<Disabled>>,
    retained: TreeTiers,
) -> TreeTiers {
    let mut applied = retained;
    for (child, tier, view_visibility, disabled) in states.iter_many(children) {
        match (tier.is_retained(retained), disabled) {
            (false, false) if view_visibility.get() => {
                let deferred = tier.tiers();
                applied.mask |= deferred.mask;
                applied.leaf |= deferred.leaf;
            }
            (false, false) => {
                commands.entity(child).insert(Disabled);
            }
            (true, true) => {
                commands.entity(child).queue(wake);
            }
            _ => {}
        }
    }
    applied
}

#[cfg(test)]
mod tests {
    use super::super::super::impostor::{
        tree_leaf_visibility, tree_projected_lod_visibility, tree_projected_trunk_visibility,
    };
    use super::*;
    use bevy::{camera::visibility::VisibilityRange, ecs::system::RunSystemOnce};

    fn distances() -> impl Iterator<Item = f32> {
        (0..6_000).map(|step| step as f32 * 0.05)
    }

    fn retained(distance: f32) -> TreeTiers {
        let requested = TreeTiers {
            mask: tree_streaming_mask(distance),
            leaf: tree_leaf_mask(
                tree_streaming_mask(distance),
                tree_streamed_leaf_representation(distance),
            ),
        };
        TreeTiers::retained(distance, &TreeLodRenderOverride::default(), requested)
    }

    fn assert_awake_while_visible(child: StreamedTreeChild, range: &VisibilityRange) {
        for distance in distances() {
            if range.is_visible_at_all(distance) {
                assert!(
                    child.is_retained(retained(distance)),
                    "{child:?} dormant at {distance} inside its range"
                );
            }
        }
    }

    #[test]
    fn streaming_keeps_adjacent_tree_lods_resident_through_every_handoff() {
        for (distance, required_bits) in [
            (7.0, (1 << 1) | (1 << 2)),
            (14.0, (1 << 2) | (1 << 3)),
            (28.0, (1 << 3) | (1 << 4)),
            (55.0, (1 << 4) | (1 << 5)),
        ] {
            assert_eq!(tree_streaming_mask(distance) & required_bits, required_bits);
        }
    }

    #[test]
    fn streaming_prefetches_each_incoming_crown_one_outgoing_band_early() {
        for (distance, incoming_lod_bit) in [
            (90.0, 1 << 4),
            (72.0, 1 << 3),
            (42.0, 1 << 2),
            (22.0, 1 << 1),
        ] {
            assert_ne!(tree_streaming_mask(distance) & incoming_lod_bit, 0);
        }
        assert_eq!(tree_streaming_mask(120.0), 1 << 5);
    }

    #[test]
    fn leaf_residency_only_overlaps_near_the_representation_handoff() {
        assert_eq!(
            tree_streamed_leaf_representation(1.4),
            Some(TreeLeafRepresentation::TexturedMesh)
        );
        assert_eq!(tree_streamed_leaf_representation(2.0), None);
        assert_eq!(
            tree_streamed_leaf_representation(2.6),
            Some(TreeLeafRepresentation::AlphaCard)
        );
    }

    #[test]
    fn no_child_is_dormant_inside_its_visibility_range() {
        for mid in [false, true] {
            assert_awake_while_visible(
                StreamedTreeChild::TRUNK,
                &tree_projected_trunk_visibility(mid, 1.0),
            );
        }
        for radius in [0.5, 3.5, 12.0] {
            assert_awake_while_visible(
                StreamedTreeChild::DETAILED_CROWN,
                &tree_projected_lod_visibility(0, 1.0, radius),
            );
            for lod in 1..=4 {
                assert_awake_while_visible(
                    StreamedTreeChild::aggregate(lod),
                    &tree_projected_lod_visibility(lod, 1.0, radius),
                );
            }
            assert_awake_while_visible(
                StreamedTreeChild::TEXTURED_LEAVES,
                &tree_leaf_visibility(TreeLeafRepresentation::TexturedMesh, 1.0, radius),
            );
            assert_awake_while_visible(
                StreamedTreeChild::LEAF_CARDS,
                &tree_leaf_visibility(TreeLeafRepresentation::AlphaCard, 1.0, radius),
            );
        }
    }

    #[test]
    fn distant_trees_retire_near_tiers() {
        let far = retained(150.0);
        assert!(StreamedTreeChild::aggregate(4).is_retained(far));
        for near in [
            StreamedTreeChild::TRUNK,
            StreamedTreeChild::DETAILED_CROWN,
            StreamedTreeChild::TEXTURED_LEAVES,
            StreamedTreeChild::aggregate(1),
            StreamedTreeChild::aggregate(2),
        ] {
            assert!(!near.is_retained(far), "{near:?} awake at 150");
        }
        assert!(!StreamedTreeChild::aggregate(4).is_retained(retained(5.0)));
    }

    #[test]
    fn a_forced_lod_keeps_every_child_awake() {
        let forced = TreeLodRenderOverride {
            lod: Some(0),
            ..default()
        };
        let none = TreeTiers { mask: 0, leaf: 0 };
        assert_eq!(TreeTiers::retained(500.0, &forced, none), TreeTiers::ALL);
    }

    fn apply(world: &mut World, parent: Entity, retained: TreeTiers) -> TreeTiers {
        let applied = world
            .run_system_once(
                move |mut commands: Commands,
                      children: Query<&Children>,
                      states: Query<StreamedTreeChildState, Allow<Disabled>>| {
                    apply_tree_child_dormancy(
                        &mut commands,
                        children.get(parent).unwrap(),
                        &states,
                        retained,
                    )
                },
            )
            .unwrap();
        world.flush();
        applied
    }

    #[test]
    fn retired_tiers_sleep_and_returning_tiers_wake() {
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let card = world
            .spawn((
                StreamedTreeChild::aggregate(4),
                ViewVisibility::HIDDEN,
                ChildOf(parent),
            ))
            .id();
        let trunk = world
            .spawn((
                StreamedTreeChild::TRUNK,
                ViewVisibility::HIDDEN,
                ChildOf(parent),
            ))
            .id();
        let leaves = world
            .spawn((
                StreamedTreeChild::TEXTURED_LEAVES,
                ViewVisibility::HIDDEN,
                ChildOf(parent),
            ))
            .id();
        let near = TreeTiers {
            mask: 0b11,
            leaf: ALPHA_CARD_LEAF_BIT,
        };
        assert_eq!(apply(&mut world, parent, near), near);
        assert!(world.entity(card).contains::<Disabled>());
        assert!(!world.entity(trunk).contains::<Disabled>());
        assert!(world.entity(leaves).contains::<Disabled>());

        apply(&mut world, parent, TreeTiers::ALL);
        for child in [card, trunk, leaves] {
            assert!(!world.entity(child).contains::<Disabled>());
        }
    }

    #[test]
    fn a_child_drawn_last_frame_stays_awake_and_is_retried() {
        let mut world = World::new();
        let parent = world.spawn_empty().id();
        let card = world
            .spawn((
                StreamedTreeChild::aggregate(4),
                ViewVisibility::VISIBLE,
                ChildOf(parent),
            ))
            .id();
        let near = TreeTiers { mask: 1, leaf: 0 };
        let applied = apply(&mut world, parent, near);
        assert!(!world.entity(card).contains::<Disabled>());
        assert_ne!(applied, near);
    }
}
