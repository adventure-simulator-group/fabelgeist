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

pub(super) struct ServicePlacement {
    pub placed: Vec<CandidateLot>,
    pub unplaced: Vec<BuildingDemand>,
}

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
        self.footprint_metres.metres()
    }
}

pub(super) fn place_services(
    seed: fabelgeist_determinism::Seed,
    population: ResidentCount,
    blocks: &[CityBlock],
    candidates: &[CandidateLot],
    demand: &[BuildingDemand],
) -> GeometryResult<ServicePlacement> {
    let blocks = blocks
        .iter()
        .copied()
        .map(|block| (block.key(), block))
        .collect::<BTreeMap<_, _>>();
    let radius = adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
        ((population.get() as f32 / REFERENCE_CITY_POPULATION).sqrt()
            * REFERENCE_CITY_RADIUS_METRES)
            .max(SERVICE_SPREAD_METRES),
    )?;
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
        for first in request_choices(seed, radius, &blocks, candidates, demand[cursor], &placed)? {
            if !fits_placed(first, &placed)? {
                continue;
            }
            placed.push(first);
            for &request in &demand[cursor + 1..end] {
                let mut chosen = None;
                for candidate in
                    request_choices(seed, radius, &blocks, candidates, request, &placed)?
                {
                    if fits_placed(candidate, &placed)? {
                        chosen = Some(candidate);
                        break;
                    }
                }
                if let Some(candidate) = chosen {
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
    Ok(ServicePlacement { placed, unplaced })
}

fn request_choices(
    seed: fabelgeist_determinism::Seed,
    radius: adventuresim_building_generator::spatial_geometry::PositiveLength,
    blocks: &BTreeMap<BlockId, CityBlock>,
    candidates: &[CandidateLot],
    request: BuildingDemand,
    placed: &[CandidateLot],
) -> GeometryResult<Vec<CandidateLot>> {
    let key = SERVICE_SITING_DOMAIN.seed(
        seed,
        &[request.usage() as u64, u64::from(request.ordinal())],
    );
    let district = request.usage().definition().district;
    let mut program = BuildingProgram::settlement(
        settlement_archetype(request.usage()),
        Some(request.usage()),
        fabelgeist_determinism::Seed::from_u64(0),
    );
    if let Some(size) = ServiceBuildingSize::for_demand(request) {
        program = program.with_service_size(size);
    }
    let footprint = program.plot_dimensions_metres();
    let mut choices = Vec::new();
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.position == LotPosition::StreetFrontage)
    {
        let candidate =
            service_candidate(*candidate, request, PlanDimensions::from_metres(footprint)?)?;
        if inside_block(candidate, blocks[&candidate.block_key])?
            && parish_siting_distance(request, candidate.lot.centre_metres, placed, radius)
                .is_some()
        {
            choices.push(candidate);
        }
    }
    // Seeded ranks are expensive. Cache each complete key once while retaining
    // the stable ordering of equal ranks and the exact accepted property sites.
    choices.sort_by_cached_key(|candidate| {
        if let Some(distance) =
            parish_siting_distance(request, candidate.lot.centre_metres, placed, radius)
            && matches!(request, BuildingDemand::Parish { .. })
        {
            return (
                (distance * 100.0) as u32,
                StreamId::new("city.service-lot-rank")
                    .rng(key, &[candidate.lot.id.0])
                    .next_u64(),
                candidate.lot.id,
            );
        }
        let distance = candidate.lot.centre_metres.metres().length();
        let target = match district {
            BuildingDistrict::Market => 0.0,
            BuildingDistrict::Edge => radius.metres(),
            BuildingDistrict::Neighbourhood | BuildingDistrict::Craft => {
                radius.metres()
                    * StreamId::new("city.service-district-radius")
                        .rng(key, &[])
                        .inclusive_unit_f32()
            }
        };
        let band = ((distance - target).abs() / SERVICE_SPREAD_METRES) as u32;
        (
            band,
            StreamId::new("city.service-lot-rank")
                .rng(key, &[candidate.lot.id.0])
                .next_u64(),
            candidate.lot.id,
        )
    });
    Ok(choices)
}

/// Linked support buildings stay near their church; neighbourhood churches
/// spread through the developed radius instead of repeating a radial lottery.
fn parish_siting_distance(
    request: BuildingDemand,
    centre: ScenePlanPoint,
    placed: &[CandidateLot],
    radius: adventuresim_building_generator::spatial_geometry::PositiveLength,
) -> Option<f32> {
    // Native scene-plane distance/rank kernel. Keep the original metre
    // arithmetic after accepting the shared position and positive radius.
    let centre = centre.metres();
    let radius = radius.metres();
    let BuildingDemand::Parish { parish, role } = request else {
        return Some(0.0);
    };
    if !matches!(role, ParishBuildingRole::Church(_)) {
        let church = placed.iter().find(|candidate| matches!(candidate.lot.service,
            Some(BuildingDemand::Parish { parish: owner, role: ParishBuildingRole::Church(_) }) if owner == parish))?;
        let distance = centre.distance(church.lot.centre_metres.metres());
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
        .map(|candidate| centre.distance(candidate.lot.centre_metres.metres()))
        .reduce(f32::min);
    Some(separation.map_or(centre.length(), |distance| {
        (radius * 2.0 - distance).max(0.0)
    }))
}

fn service_candidate(
    mut candidate: CandidateLot,
    request: BuildingDemand,
    footprint: PlanDimensions,
) -> GeometryResult<CandidateLot> {
    let old_depth = candidate.lot.dimensions_metres().y;
    candidate.lot.service = Some(request);
    candidate.lot.footprint_metres = footprint;
    let new_depth = candidate.lot.dimensions_metres().y;
    candidate.lot.centre_metres =
        candidate
            .lot
            .centre_metres
            .translated(PlanDisplacement::try_from(
                candidate.lot.orientation.local_to_world(Vec2::Y)
                    * ((new_depth - old_depth) * 0.5 + SERVICE_EDGE_CLEARANCE_METRES
                        - ORDINARY_STREET_HALF_WIDTH_METRES),
            )?)?;
    Ok(candidate)
}

fn inside_block(candidate: CandidateLot, block: CityBlock) -> GeometryResult<bool> {
    plots::inside_block(candidate.lot, block)
}

fn fits_placed(candidate: CandidateLot, placed: &[CandidateLot]) -> GeometryResult<bool> {
    let proposed = plots::reservation(candidate.lot)?;
    for other in placed {
        if plots::lots_overlap(proposed, plots::reservation(other.lot)?) {
            return Ok(false);
        }
    }
    Ok(true)
}
