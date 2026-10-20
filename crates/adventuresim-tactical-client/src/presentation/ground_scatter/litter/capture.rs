//! Select nearby rendered litter roots independently of mesh batch origins.
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::{animation::NonnegativeLength, scene_coordinates::Scene};
use bevy::math::Vec2;
use std::collections::BTreeMap;

/// Real generated litter placements retained independently of batched mesh origins.
/// Capture diagnostics use this pair to frame dry leaves and twigs without
/// mistaking a shared batch-cell transform for the rendered subjects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GroundLitterCapturePair {
    pub(crate) dry_leaf: Position<Scene>,
    pub(crate) twig: Position<Scene>,
}

struct PairCandidate {
    separation: NonnegativeLength,
    pair: GroundLitterCapturePair,
}

impl GroundLitterCapturePair {
    pub(crate) const MAXIMUM_SEPARATION: NonnegativeLength =
        match NonnegativeLength::from_metres(0.55) {
            Ok(distance) => distance,
            Err(_) => {
                panic!("the authored litter capture separation must be finite and nonnegative")
            }
        };

    pub(in crate::presentation::ground_scatter::litter) fn closest(
        leaf_positions: &[Position<Scene>],
        twig_positions: &[Position<Scene>],
        maximum_separation: NonnegativeLength,
        maximum_pairs: usize,
    ) -> Vec<Self> {
        if maximum_separation == NonnegativeLength::ZERO || maximum_pairs == 0 {
            return Vec::new();
        }
        // The native spatial hash and distance sort operate in scene metres;
        // candidates retain checked roots and lengths outside those equations.
        let cell_size = maximum_separation.metres();
        let cell = |position: Position<Scene>| {
            let position = position.metres();
            (
                (position.x / cell_size).floor() as i32,
                (position.z / cell_size).floor() as i32,
            )
        };
        let mut leaves_by_cell = BTreeMap::<(i32, i32), Vec<Position<Scene>>>::new();
        for &leaf in leaf_positions {
            leaves_by_cell.entry(cell(leaf)).or_default().push(leaf);
        }
        let mut candidates = Vec::<PairCandidate>::new();
        for &twig in twig_positions {
            let (cell_x, cell_z) = cell(twig);
            for dz in -1..=1 {
                for dx in -1..=1 {
                    let Some(leaves) = leaves_by_cell.get(&(cell_x + dx, cell_z + dz)) else {
                        continue;
                    };
                    for &leaf in leaves {
                        let leaf_metres = leaf.metres();
                        let twig_metres = twig.metres();
                        let distance = Vec2::new(leaf_metres.x, leaf_metres.z)
                            .distance(Vec2::new(twig_metres.x, twig_metres.z));
                        if distance > cell_size {
                            continue;
                        }
                        let Ok(separation) = NonnegativeLength::from_arithmetic(distance) else {
                            continue;
                        };
                        candidates.push(PairCandidate {
                            separation,
                            pair: Self {
                                dry_leaf: leaf,
                                twig,
                            },
                        });
                    }
                }
            }
        }
        candidates.sort_by(|left, right| {
            left.separation
                .metres()
                .total_cmp(&right.separation.metres())
                .then_with(|| {
                    left.pair
                        .dry_leaf
                        .metres()
                        .x
                        .total_cmp(&right.pair.dry_leaf.metres().x)
                })
                .then_with(|| {
                    left.pair
                        .dry_leaf
                        .metres()
                        .z
                        .total_cmp(&right.pair.dry_leaf.metres().z)
                })
                .then_with(|| {
                    left.pair
                        .twig
                        .metres()
                        .x
                        .total_cmp(&right.pair.twig.metres().x)
                })
                .then_with(|| {
                    left.pair
                        .twig
                        .metres()
                        .z
                        .total_cmp(&right.pair.twig.metres().z)
                })
        });
        candidates
            .into_iter()
            .take(maximum_pairs)
            .map(|candidate| candidate.pair)
            .collect()
    }
}
