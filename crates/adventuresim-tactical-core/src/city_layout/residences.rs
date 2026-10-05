//! Pack street rows around the already reserved civic and workplace properties.
use super::*;
use std::collections::BTreeMap;

mod roster;
pub(super) use roster::SelectedRoster;

const FRONTAGE_SEARCH_STEP_METRES: f32 = 1.0;

pub(super) fn pack(
    seed: u64,
    blocks: &[CityBlock],
    extent: DevelopmentExtent,
    services: Vec<CandidateLot>,
) -> Vec<CandidateLot> {
    let mut properties = BTreeMap::<BlockId, Vec<CandidateLot>>::new();
    for service in services {
        properties
            .entry(service.block_key)
            .or_default()
            .push(service);
    }
    for block in blocks
        .iter()
        .copied()
        .filter(|b| !b.is_market() && block_is_inside_city(*b, extent))
    {
        let accepted = properties.entry(block.id).or_default();
        let mut edges = [0, 1, 2, 3];
        edges.sort_by(|&a, &b| {
            let length =
                |edge: usize| block.corners[edge].distance_squared(block.corners[(edge + 1) % 4]);
            length(b).total_cmp(&length(a)).then(a.cmp(&b))
        });
        for edge in edges {
            pack_frontage(seed, block, edge, accepted);
        }
    }
    properties.into_values().flatten().collect()
}

fn pack_frontage(seed: u64, block: CityBlock, edge: usize, accepted: &mut Vec<CandidateLot>) {
    let start = block.corners[edge];
    let end = block.corners[(edge + 1) % 4];
    let tangent = (end - start).normalize();
    let mut row = Vec::new();
    append_frontage(
        &mut row,
        seed,
        StreamId::new("city.frontage-identity")
            .seed(block.id.0, &[edge as u64])
            .to_u64(),
        block.id,
        start,
        end,
        block.streets[edge].half_width(),
    );
    let mut cursor = FRONTAGE_CORNER_CLEARANCE_METRES;
    let end = start.distance(end) - FRONTAGE_CORNER_CLEARANCE_METRES;
    for candidate in row {
        let reservation = plots::reservation(candidate.lot);
        let left = plots::corners(reservation)
            .into_iter()
            .map(|point| (point - start).dot(tangent))
            .fold(f32::INFINITY, f32::min);
        let width = reservation.footprint_metres.x;
        let mut position = cursor;
        while position + width <= end {
            let mut trial = candidate;
            trial.lot.centre_metres += tangent * (position - left);
            if plots::inside_block(trial.lot, block)
                && accepted.iter().all(|other| {
                    !plots::lots_overlap(
                        plots::reservation(trial.lot),
                        plots::reservation(other.lot),
                    )
                })
            {
                accepted.push(trial);
                cursor = position + width + PARTY_WALL_CLEARANCE_METRES;
                break;
            }
            position += FRONTAGE_SEARCH_STEP_METRES;
        }
        // If a deep property cannot fit, the next household can still use the
        // remaining frontage. No accepted court or side passage is reduced.
    }
}
