//! Reject a pose only when a measured geometric pair blocks every neighbour pose.
use super::*;
use search::BlockSearch;

impl BlockSearch<'_> {
    pub(super) fn propagate_domains(
        &self,
        candidates: &mut [DomainChoices],
    ) -> Result<(), CoupledPackingIssue> {
        // A bounded propagation pass only removes impossible choices. Stopping
        // early can leave extra search work; it never accepts a collision.
        for _ in 0..self.domains.len().saturating_mul(self.domains.len()) {
            let previous = candidates.to_vec();
            self.propagate_frontage_capacity(candidates)?;
            for first in 0..candidates.len() {
                for second in 0..candidates.len() {
                    if first == second || candidates[second].intervals.is_empty() {
                        continue;
                    }
                    let a = &self.domains[candidates[first].domain.index()];
                    let b = &self.domains[candidates[second].domain.index()];
                    for forbidden in a.unavoidable_conflicts(b, &candidates[second].intervals)? {
                        candidates[first].intervals = candidates[first]
                            .intervals
                            .iter()
                            .flat_map(|interval| interval.without(forbidden))
                            .collect();
                    }
                }
            }
            if candidates
                .iter()
                .any(|choices| choices.intervals.is_empty())
                || unchanged(&previous, candidates)
            {
                break;
            }
        }
        Ok(())
    }
}
impl PlacementDomain {
    fn unavoidable_conflicts(
        &self,
        other: &Self,
        ranges: &[FrontageInterval],
    ) -> Result<Vec<FrontageInterval>, CoupledPackingIssue> {
        let at = |frontage_displacement| {
            self.proposed.conflicts(
                &other.proposed,
                self.frontage.tangent(),
                other.frontage.tangent().as_dvec2() * frontage_displacement,
            )
        };
        let mut unavoidable: Option<Vec<Option<FrontageInterval>>> = None;
        for range in ranges {
            let endpoints = at(range.minimum_metres())
                .map_err(|_| CoupledPackingIssue::NumericalFailure)?
                .into_iter()
                .zip(
                    at(range.maximum_metres())
                        .map_err(|_| CoupledPackingIssue::NumericalFailure)?,
                )
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
        Ok(unavoidable
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect())
    }
}
fn unchanged(previous: &[DomainChoices], current: &[DomainChoices]) -> bool {
    previous
        .iter()
        .zip(current)
        .all(|(first_choices, second_choices)| {
            let a = &first_choices.intervals;
            let b = &second_choices.intervals;
            a.len() == b.len()
                && a.iter().zip(b).all(|(first, second)| {
                    (first.minimum_metres() - second.minimum_metres()).abs()
                        <= CityPlotBounds::COORDINATE_TOLERANCE_METRES
                        && (first.maximum_metres() - second.maximum_metres()).abs()
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
        let generated = CitySite::central_german_market_town()
            .unwrap()
            .generate(
                (42).into(),
                adventuresim_core::settlement_property::ResidentCount::new(900),
                &economy(),
            )
            .unwrap();
        let mut frontage = *generated
            .packing
            .as_ref()
            .unwrap()
            .frontages
            .values()
            .next()
            .unwrap();
        frontage.block.corners = [
            Vec2::ZERO,
            Vec2::new(30.0, 0.0),
            Vec2::splat(30.0),
            Vec2::new(0.0, 30.0),
        ]
        .map(|point| ScenePlanPoint::try_from(point).unwrap());
        let domain = |edge: usize, half_range: f64| {
            let frontage = ParcelFrontage::on_edge(frontage.lot, frontage.block, edge).unwrap();
            let bounds = CityPlotBounds::new(
                crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::splat(10.0)).unwrap(),
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    Vec2::splat(4.0),
                )
                .unwrap(),
                BuildingOrientation::from_radians(0.0).unwrap(),
            )
            .unwrap();
            PlacementDomain {
                owner: CityPropertyId(edge as u64 + 1),
                base_translation: PlanDisplacement::ZERO,
                proposed: ParcelGeometry {
                    reservation: bounds,
                    buildings: vec![bounds],
                    bearings: Vec::new(),
                    garden: None,
                },
                frontage,
                allowed: FrontageInterval::new(-half_range, half_range).unwrap(),
            }
        };
        let a = domain(0, 8.0);
        let constrained = domain(3, 0.5);
        let forbidden = a
            .unavoidable_conflicts(&constrained, &[constrained.allowed])
            .unwrap();
        assert!(
            !forbidden.is_empty(),
            "a corner occupying every neighbour pose must be pruned"
        );
        assert!(
            forbidden
                .iter()
                .all(|range| range.minimum_metres() < 0.0 && range.maximum_metres() > 0.0)
        );
        let clear_choice = domain(3, 8.0);
        assert!(
            a.unavoidable_conflicts(&clear_choice, &[clear_choice.allowed])
                .unwrap()
                .is_empty(),
            "a neighbour can clear the corner; retain that branch"
        );
        let disjoint_choices = [
            FrontageInterval::new(-8.0, -7.0).unwrap(),
            FrontageInterval::new(-0.5, 0.5).unwrap(),
        ];
        assert!(
            a.unavoidable_conflicts(&clear_choice, &disjoint_choices)
                .unwrap()
                .is_empty(),
            "one colliding interval must not erase another clear neighbour choice"
        );
    }
}
