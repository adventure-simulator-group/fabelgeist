//! Reject a pose only when a measured geometric pair blocks every neighbour pose.
use super::*;
use search::BlockSearch;

impl BlockSearch<'_> {
    pub(super) fn propagate_domains(&self, candidates: &mut [(usize, Vec<FrontageInterval>)]) {
        // A bounded propagation pass only removes impossible choices. Stopping
        // early can leave extra search work; it never accepts a collision.
        for _ in 0..self.domains.len().saturating_mul(self.domains.len()) {
            let previous = candidates.to_vec();
            self.propagate_frontage_capacity(candidates);
            for first in 0..candidates.len() {
                for second in 0..candidates.len() {
                    if first == second || candidates[second].1.is_empty() {
                        continue;
                    }
                    let a = &self.domains[candidates[first].0];
                    let b = &self.domains[candidates[second].0];
                    for forbidden in a.unavoidable_conflicts(b, &candidates[second].1) {
                        candidates[first].1 = candidates[first]
                            .1
                            .iter()
                            .flat_map(|interval| interval.without(forbidden))
                            .collect();
                    }
                }
            }
            if candidates.iter().any(|(_, intervals)| intervals.is_empty())
                || unchanged(&previous, candidates)
            {
                break;
            }
        }
    }
}
impl PlacementDomain {
    fn unavoidable_conflicts(
        &self,
        other: &Self,
        ranges: &[FrontageInterval],
    ) -> Vec<FrontageInterval> {
        let at = |position| {
            self.proposed.conflicts(
                &other.proposed,
                self.frontage.tangent(),
                other.frontage.tangent().as_dvec2() * position,
            )
        };
        let mut unavoidable: Option<Vec<Option<FrontageInterval>>> = None;
        for range in ranges {
            let endpoints = at(range.minimum_metres)
                .into_iter()
                .zip(at(range.maximum_metres))
                .map(|(a, b)| a.and_then(|first| b.and_then(|second| first.intersection(second))))
                .collect::<Vec<_>>();
            unavoidable = Some(match unavoidable {
                None => endpoints,
                Some(previous) => previous
                    .into_iter()
                    .zip(endpoints)
                    .map(|(a, b)| {
                        a.and_then(|first| b.and_then(|second| first.intersection(second)))
                    })
                    .collect(),
            });
        }
        unavoidable
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect()
    }
}
fn unchanged(
    previous: &[(usize, Vec<FrontageInterval>)],
    current: &[(usize, Vec<FrontageInterval>)],
) -> bool {
    previous.iter().zip(current).all(|((_, a), (_, b))| {
        a.len() == b.len()
            && a.iter().zip(b).all(|(first, second)| {
                (first.minimum_metres - second.minimum_metres).abs()
                    <= CityPlotBounds::COORDINATE_TOLERANCE_METRES
                    && (first.maximum_metres - second.maximum_metres).abs()
                        <= CityPlotBounds::COORDINATE_TOLERANCE_METRES
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city_layout::{CitySite, tests::economy};

    #[test]
    fn cross_frontage_propagation_preserves_clear_neighbor_choices() {
        let generated = CitySite::central_german_market_town().generate(42, 900, &economy());
        let mut frontage = *generated.packing.frontages.values().next().unwrap();
        frontage.block.corners = [
            Vec2::ZERO,
            Vec2::new(30.0, 0.0),
            Vec2::splat(30.0),
            Vec2::new(0.0, 30.0),
        ];
        let domain = |edge: usize, half_range: f64| {
            let mut frontage = frontage;
            frontage.edge = edge;
            let bounds = CityPlotBounds {
                centre_metres: Vec2::splat(10.0),
                dimensions_metres: Vec2::splat(4.0),
                orientation: BuildingOrientation::from_radians(0.0).unwrap(),
            };
            PlacementDomain {
                owner: CityPropertyId(edge as u64 + 1),
                base_translation_metres: Vec2::ZERO,
                proposed: ParcelGeometry {
                    reservation: bounds,
                    buildings: vec![bounds],
                    bearings: Vec::new(),
                    garden: None,
                },
                frontage,
                allowed: FrontageInterval {
                    minimum_metres: -half_range,
                    maximum_metres: half_range,
                },
            }
        };
        let a = domain(0, 8.0);
        let constrained = domain(3, 0.5);
        let forbidden = a.unavoidable_conflicts(&constrained, &[constrained.allowed]);
        assert!(
            !forbidden.is_empty(),
            "a corner occupying every neighbour pose must be pruned"
        );
        assert!(
            forbidden
                .iter()
                .all(|range| range.minimum_metres < 0.0 && range.maximum_metres > 0.0)
        );
        let clear_choice = domain(3, 8.0);
        assert!(
            a.unavoidable_conflicts(&clear_choice, &[clear_choice.allowed])
                .is_empty(),
            "a neighbour can clear the corner; retain that branch"
        );
        let disjoint_choices = [
            FrontageInterval {
                minimum_metres: -8.0,
                maximum_metres: -7.0,
            },
            FrontageInterval {
                minimum_metres: -0.5,
                maximum_metres: 0.5,
            },
        ];
        assert!(
            a.unavoidable_conflicts(&clear_choice, &disjoint_choices)
                .is_empty(),
            "one colliding interval must not erase another clear neighbour choice"
        );
    }
}
