//! Reserve service lots before filling remaining frontage with residents.
use super::*;
use adventuresim_building_generator::{
    BuildingArchetype, BuildingProgram, ServiceBuildingSize, settlement_archetype,
};
use adventuresim_world_schema::settlement_buildings::{BuildingDistrict, ParishBuildingRole};
use std::collections::BTreeMap;

const SERVICE_SITING_DOMAIN: StreamId = StreamId::new("city.service-siting");
pub(super) const SERVICE_EDGE_CLEARANCE_METRES: f32 = PRIMARY_STREET_HALF_WIDTH_METRES + 0.5;
const SERVICE_SPREAD_METRES: f32 = 80.0;
const REFERENCE_CITY_POPULATION: f32 = 40_000.0;
const REFERENCE_CITY_RADIUS_METRES: f32 = 500.0;

impl CityBuildingLot {
    pub fn building_use(self) -> Option<BuildingUse> {
        self.service.map(|demand| demand.usage())
    }

    pub fn archetype(self) -> BuildingArchetype {
        self.building_use()
            .map(settlement_archetype)
            .unwrap_or_else(|| self.house_class.archetype())
    }

    pub fn service_size(self) -> Option<adventuresim_building_generator::ServiceBuildingSize> {
        self.service.and_then(ServiceBuildingSize::for_demand)
    }

    pub fn dimensions_metres(self) -> Vec2 {
        self.footprint_metres
    }
}

pub(super) fn place_services(
    seed: u64,
    population: u32,
    blocks: &[CityBlock],
    candidates: &[CandidateLot],
    demand: &[BuildingDemand],
) -> (Vec<CandidateLot>, Vec<BuildingDemand>) {
    let blocks = blocks
        .iter()
        .copied()
        .map(|block| (block.key(), block))
        .collect::<BTreeMap<_, _>>();
    let radius = ((population as f32 / REFERENCE_CITY_POPULATION).sqrt()
        * REFERENCE_CITY_RADIUS_METRES)
        .max(SERVICE_SPREAD_METRES);
    let mut placed = Vec::<CandidateLot>::new();
    let mut unplaced = Vec::new();
    let mut cursor = 0;
    while cursor < demand.len() {
        let end = match demand[cursor] {
            BuildingDemand::Parish { parish, .. } => cursor + demand[cursor..].iter()
                .take_while(|request| matches!(request, BuildingDemand::Parish { parish: owner, .. } if *owner == parish)).count(),
            _ => cursor + 1,
        };
        let before = placed.len();
        let mut accepted = false;
        for first in request_choices(seed, radius, &blocks, candidates, demand[cursor], &placed) {
            if !fits_placed(first, &placed) {
                continue;
            }
            placed.push(first);
            for &request in &demand[cursor + 1..end] {
                if let Some(candidate) =
                    request_choices(seed, radius, &blocks, candidates, request, &placed)
                        .into_iter()
                        .find(|candidate| fits_placed(*candidate, &placed))
                {
                    placed.push(candidate);
                } else {
                    break;
                }
            }
            if placed.len() - before == end - cursor {
                accepted = true;
                break;
            }
            placed.truncate(before);
        }
        if !accepted {
            unplaced.extend_from_slice(&demand[cursor..end]);
        }
        cursor = end;
    }
    (placed, unplaced)
}

fn request_choices(
    seed: u64,
    radius: f32,
    blocks: &BTreeMap<BlockId, CityBlock>,
    candidates: &[CandidateLot],
    request: BuildingDemand,
    placed: &[CandidateLot],
) -> Vec<CandidateLot> {
    let key = SERVICE_SITING_DOMAIN
        .seed(
            seed,
            &[request.usage() as u64, u64::from(request.ordinal())],
        )
        .to_u64();
    let district = request.usage().definition().district;
    let mut program = BuildingProgram::settlement(
        settlement_archetype(request.usage()),
        Some(request.usage()),
        0,
    );
    if let Some(size) = ServiceBuildingSize::for_demand(request) {
        program = program.with_service_size(size);
    }
    let footprint = program.plot_dimensions_metres();
    let mut choices = candidates
        .iter()
        .filter(|candidate| !candidate.rear_court)
        .map(|candidate| service_candidate(*candidate, request, footprint))
        .filter(|candidate| inside_block(*candidate, blocks[&candidate.block_key]))
        .filter(|candidate| {
            parish_siting_distance(request, candidate.lot.centre_metres, placed, radius).is_some()
        })
        .collect::<Vec<_>>();
    choices.sort_by_key(|candidate| {
        if let Some(distance) =
            parish_siting_distance(request, candidate.lot.centre_metres, placed, radius)
            && matches!(request, BuildingDemand::Parish { .. })
        {
            return (
                (distance * 100.0) as u32,
                StreamId::new("city.service-lot-rank")
                    .rng(key, &[candidate.lot.id])
                    .next_u64(),
                candidate.lot.id,
            );
        }
        let distance = candidate.lot.centre_metres.length();
        let target = match district {
            BuildingDistrict::Market => 0.0,
            BuildingDistrict::Edge => radius,
            BuildingDistrict::Neighbourhood | BuildingDistrict::Craft => {
                radius
                    * StreamId::new("city.service-district-radius")
                        .rng(key, &[])
                        .inclusive_unit_f32()
            }
        };
        let band = ((distance - target).abs() / SERVICE_SPREAD_METRES) as u32;
        (
            band,
            StreamId::new("city.service-lot-rank")
                .rng(key, &[candidate.lot.id])
                .next_u64(),
            candidate.lot.id,
        )
    });
    choices
}

/// Linked support buildings stay near their church; neighbourhood churches
/// spread through the developed radius instead of repeating a radial lottery.
fn parish_siting_distance(
    request: BuildingDemand,
    centre: Vec2,
    placed: &[CandidateLot],
    radius: f32,
) -> Option<f32> {
    let BuildingDemand::Parish { parish, role } = request else {
        return Some(0.0);
    };
    if !matches!(role, ParishBuildingRole::Church(_)) {
        let church = placed.iter().find(|candidate| matches!(candidate.lot.service,
            Some(BuildingDemand::Parish { parish: owner, role: ParishBuildingRole::Church(_) }) if owner == parish))?;
        let distance = centre.distance(church.lot.centre_metres);
        return (distance <= CITY_PARISH_PRECINCT_RADIUS_METRES).then_some(distance);
    }
    if centre.length() > radius {
        return None;
    }
    let separation = placed
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.lot.service,
                Some(BuildingDemand::Parish {
                    role: ParishBuildingRole::Church(_),
                    ..
                })
            )
        })
        .map(|candidate| centre.distance(candidate.lot.centre_metres))
        .reduce(f32::min);
    Some(separation.map_or(centre.length(), |distance| {
        (radius * 2.0 - distance).max(0.0)
    }))
}

fn service_candidate(
    mut candidate: CandidateLot,
    request: BuildingDemand,
    footprint: Vec2,
) -> CandidateLot {
    let old_depth = candidate.lot.dimensions_metres().y;
    candidate.lot.service = Some(request);
    candidate.lot.footprint_metres = footprint;
    let new_depth = candidate.lot.dimensions_metres().y;
    candidate.lot.centre_metres += candidate.lot.orientation.local_to_world(Vec2::Y)
        * ((new_depth - old_depth) * 0.5 + SERVICE_EDGE_CLEARANCE_METRES
            - ORDINARY_STREET_HALF_WIDTH_METRES);
    candidate
}

fn inside_block(candidate: CandidateLot, block: CityBlock) -> bool {
    plots::inside_block(candidate.lot, block)
}

fn fits_placed(candidate: CandidateLot, placed: &[CandidateLot]) -> bool {
    placed.iter().all(|other| {
        !plots::lots_overlap(
            plots::reservation(candidate.lot),
            plots::reservation(other.lot),
        )
    })
}
