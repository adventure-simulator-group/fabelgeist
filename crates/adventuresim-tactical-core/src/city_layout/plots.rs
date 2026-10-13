//! Plot ownership reserves the side passage and rear court before packing buildings.
use super::*;

pub(super) const SIDE_PASSAGE_METRES: f32 = 2.0;
pub(super) const REAR_COURT_METRES: f32 = 6.0;
pub(super) const STREET_EDGE_TOLERANCE_METRES: f32 = 0.01;

pub(super) fn reservation(lot: CityBuildingLot) -> GeometryResult<CityPlotBounds> {
    let apron = if lot.service.is_some() {
        services::SERVICE_EDGE_CLEARANCE_METRES - ORDINARY_STREET_HALF_WIDTH_METRES
    } else {
        0.0
    };
    let (rear, margin) = if lot.has_rear_range() {
        (
            REAR_COURT_METRES + compound::REAR_RANGE_DEPTH_METRES,
            compound::COMPOUND_EDGE_MARGIN_METRES,
        )
    } else {
        (REAR_COURT_METRES, 0.0)
    };
    let extra = Vec2::new(
        SIDE_PASSAGE_METRES + margin * 2.0,
        rear + apron + margin * 2.0,
    );
    CityPlotBounds::new(
        lot.centre_metres.translated(PlanDisplacement::try_from(
            lot.orientation.local_to_world(Vec2::new(
                lot.passage_side.sign() * SIDE_PASSAGE_METRES * 0.5,
                (rear - apron) * 0.5,
            )),
        )?)?,
        PlanDimensions::from_metres(lot.footprint_metres.metres() + extra)?,
        lot.orientation,
    )
}

pub(super) fn corners(bounds: CityPlotBounds) -> [Vec2; 4] {
    bounds.corners()
}

pub(super) fn inside_block(lot: CityBuildingLot, block: CityBlock) -> GeometryResult<bool> {
    let widths = [
        block.streets[0].half_width()?,
        block.streets[1].half_width()?,
        block.streets[2].half_width()?,
        block.streets[3].half_width()?,
    ];
    Ok(corners(reservation(lot)?).into_iter().all(|point| {
        (0..4).all(|edge| {
            let start = block.corners_metres()[edge];
            let tangent = (block.corners_metres()[(edge + 1) % 4] - start).normalize();
            tangent.perp_dot(point - start) + STREET_EDGE_TOLERANCE_METRES >= widths[edge].metres()
        })
    }))
}

pub(super) fn lots_overlap(first: CityPlotBounds, second: CityPlotBounds) -> bool {
    let first_half = first.dimensions_metres() * 0.5;
    let second_half = second.dimensions_metres() * 0.5;
    let first_axes = [
        first.orientation().local_to_world(Vec2::X),
        first.orientation().local_to_world(Vec2::Y),
    ];
    let second_axes = [
        second.orientation().local_to_world(Vec2::X),
        second.orientation().local_to_world(Vec2::Y),
    ];
    let delta = second.centre_metres() - first.centre_metres();
    first_axes.into_iter().chain(second_axes).all(|axis| {
        let first_radius = first_half.x * axis.dot(first_axes[0]).abs()
            + first_half.y * axis.dot(first_axes[1]).abs();
        let second_radius = second_half.x * axis.dot(second_axes[0]).abs()
            + second_half.y * axis.dot(second_axes[1]).abs();
        delta.dot(axis).abs() < first_radius + second_radius - STREET_EDGE_TOLERANCE_METRES
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mirrored_frontages_fit_their_original_slots_before_candidate_culling() {
        let mut saw_left = false;
        for seed in [42, 47, 101].map(fabelgeist_determinism::Seed::from_u64) {
            for tangent in [Vec2::X, Vec2::new(0.8, 0.6)] {
                let mut candidates = Vec::new();
                append_frontage(
                    &mut candidates,
                    seed,
                    fabelgeist_determinism::Seed::from_u64(0),
                    BlockId(0),
                    ScenePlanPoint::ORIGIN,
                    ScenePlanPoint::try_from(tangent * 500.0).unwrap(),
                    adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                        ORDINARY_STREET_HALF_WIDTH_METRES,
                    )
                    .unwrap(),
                )
                .unwrap();
                saw_left |= candidates
                    .iter()
                    .any(|c| c.lot.passage_side == PropertySide::Left);
                for pair in candidates.windows(2) {
                    assert!(
                        !lots_overlap(
                            reservation(pair[0].lot).unwrap(),
                            reservation(pair[1].lot).unwrap()
                        ),
                        "mirroring must not consume the neighbouring frontage slot"
                    );
                }
            }
        }
        assert!(saw_left);
    }

    #[test]
    fn reserved_courts_and_passages_do_not_overlap_neighbouring_plots() {
        let city = CitySite::central_german_market_town()
            .unwrap()
            .generate(
                (42).into(),
                adventuresim_core::settlement_property::ResidentCount::new(2000),
                &SettlementEconomyProfile::stage_placeholder(),
            )
            .unwrap();
        for (index, first) in city.lots.iter().enumerate() {
            for second in &city.lots[index + 1..] {
                assert!(!lots_overlap(
                    reservation(*first).unwrap(),
                    reservation(*second).unwrap()
                ));
            }
            if first.service.is_some() {
                continue;
            }
            let passage = first.centre_metres.metres()
                + first.orientation.local_to_world(Vec2::new(
                    first.passage_side.sign()
                        * (first.footprint_metres.metres().x + SIDE_PASSAGE_METRES)
                        * 0.5,
                    -first.footprint_metres.metres().y * 0.5
                        - if first.has_rear_range() {
                            compound::COMPOUND_EDGE_MARGIN_METRES
                        } else {
                            0.0
                        },
                ));
            assert!(
                city.streets.iter().any(|street| street.contains(passage)),
                "side passage must meet the street"
            );
        }
    }
}
