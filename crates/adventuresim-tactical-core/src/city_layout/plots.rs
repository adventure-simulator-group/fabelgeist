//! Plot ownership reserves the side passage and rear court before packing buildings.
use super::*;
use std::collections::BTreeMap;

pub(super) const SIDE_PASSAGE_METRES: f32 = 2.0;
pub(super) const REAR_COURT_METRES: f32 = 6.0;
const STREET_EDGE_TOLERANCE_METRES: f32 = 0.01;

pub(super) fn reservation(lot: CityBuildingLot) -> CityBuildingLot {
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
    CityBuildingLot {
        centre_metres: lot.centre_metres
            + lot
                .orientation
                .local_to_world(Vec2::new(SIDE_PASSAGE_METRES * 0.5, (rear - apron) * 0.5)),
        footprint_metres: lot.footprint_metres + extra,
        ..lot
    }
}

pub(super) fn corners(lot: CityBuildingLot) -> [Vec2; 4] {
    let half = lot.footprint_metres * 0.5;
    [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        half,
        Vec2::new(-half.x, half.y),
    ]
    .map(|point| lot.centre_metres + lot.orientation.local_to_world(point))
}

pub(super) fn inside_block(lot: CityBuildingLot, block: CityBlock) -> bool {
    let widths = block.streets.map(StreetClass::half_width);
    corners(reservation(lot)).into_iter().all(|point| {
        (0..4).all(|edge| {
            let start = block.corners[edge];
            let tangent = (block.corners[(edge + 1) % 4] - start).normalize();
            tangent.perp_dot(point - start) + STREET_EDGE_TOLERANCE_METRES >= widths[edge]
        })
    })
}

pub(super) fn remove_overlapping_candidates(candidates: Vec<CandidateLot>) -> Vec<CandidateLot> {
    let mut accepted = Vec::<CandidateLot>::new();
    let mut buckets = BTreeMap::<(i32, i32), Vec<usize>>::new();
    for candidate in candidates {
        let plot = reservation(candidate.lot);
        let (min, max) = corners(plot).into_iter().fold(
            (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
            |(min, max), p| (min.min(p), max.max(p)),
        );
        let min = (min / SPATIAL_BUCKET_METRES).floor().as_ivec2();
        let max = (max / SPATIAL_BUCKET_METRES).floor().as_ivec2();
        let cells = (min.x..=max.x)
            .flat_map(|x| (min.y..=max.y).map(move |y| (x, y)))
            .collect::<Vec<_>>();
        if cells
            .iter()
            .filter_map(|cell| buckets.get(cell))
            .flatten()
            .any(|&index| lots_overlap(plot, reservation(accepted[index].lot)))
        {
            continue;
        }
        for cell in cells {
            buckets.entry(cell).or_default().push(accepted.len());
        }
        accepted.push(candidate);
    }
    accepted
}

pub(super) fn lots_overlap(first: CityBuildingLot, second: CityBuildingLot) -> bool {
    let first_half = first.dimensions_metres() * 0.5;
    let second_half = second.dimensions_metres() * 0.5;
    let first_axes = [
        first.orientation.local_to_world(Vec2::X),
        first.orientation.local_to_world(Vec2::Y),
    ];
    let second_axes = [
        second.orientation.local_to_world(Vec2::X),
        second.orientation.local_to_world(Vec2::Y),
    ];
    let delta = second.centre_metres - first.centre_metres;
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
    fn reserved_courts_and_passages_do_not_overlap_neighbouring_plots() {
        let city = CitySite::central_german_market_town().generate(
            42,
            2000,
            &SettlementEconomyProfile::stage_placeholder(),
        );
        for (index, first) in city.lots.iter().enumerate() {
            for second in &city.lots[index + 1..] {
                assert!(!lots_overlap(reservation(*first), reservation(*second)));
            }
            if first.service.is_some() {
                continue;
            }
            let passage = first.centre_metres
                + first.orientation.local_to_world(Vec2::new(
                    (first.footprint_metres.x + SIDE_PASSAGE_METRES) * 0.5,
                    -first.footprint_metres.y * 0.5
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
