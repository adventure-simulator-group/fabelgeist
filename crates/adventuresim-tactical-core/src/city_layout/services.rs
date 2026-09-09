//! Reserve service lots before filling remaining frontage with residents.
use super::*;
use adventuresim_building_generator::{
    BuildingArchetype, BuildingProgram, ServiceBuildingSize, settlement_archetype,
};
use adventuresim_world_schema::settlement_buildings::BuildingDistrict;

const SERVICE_SITING_DOMAIN: u64 = 0x7365_7276_6963_6573;
const SERVICE_EDGE_CLEARANCE_METRES: f32 = PRIMARY_STREET_HALF_WIDTH_METRES + 0.5;
const SERVICE_SPREAD_METRES: f32 = 80.0;
const REFERENCE_CITY_POPULATION: f32 = 40_000.0;
const REFERENCE_CITY_RADIUS_METRES: f32 = 500.0;

impl CityBuildingLot {
    pub fn building_use(self) -> Option<BuildingUse> {
        self.service.map(|demand| demand.usage)
    }

    pub fn archetype(self) -> BuildingArchetype {
        self.building_use()
            .map(settlement_archetype)
            .unwrap_or_else(|| self.house_class.archetype())
    }

    pub fn service_size(self) -> Option<adventuresim_building_generator::ServiceBuildingSize> {
        self.service.and_then(|request| {
            adventuresim_building_generator::ServiceBuildingSize::for_capacity(
                request.usage,
                request.capacity,
            )
        })
    }

    pub fn dimensions_metres(self) -> Vec2 {
        self.footprint_metres
    }
}

pub(super) fn place_services(
    seed: u64,
    population: u32,
    nodes: [[Vec2; STREET_LINE_COUNT]; STREET_LINE_COUNT],
    candidates: &[CandidateLot],
    demand: &[BuildingDemand],
) -> (Vec<CandidateLot>, Vec<BuildingDemand>) {
    let blocks = city_blocks(nodes)
        .map(|block| (block.key(), block))
        .collect::<BTreeMap<_, _>>();
    let radius = ((population as f32 / REFERENCE_CITY_POPULATION).sqrt()
        * REFERENCE_CITY_RADIUS_METRES)
        .max(SERVICE_SPREAD_METRES);
    let mut placed = Vec::<CandidateLot>::new();
    let mut unplaced = Vec::new();
    for &request in demand {
        let key = mix64(
            seed ^ SERVICE_SITING_DOMAIN
                ^ (request.usage as u64).rotate_left(23)
                ^ u64::from(request.ordinal),
        );
        let district = request.usage.definition().district;
        let mut program = BuildingProgram::settlement(
            settlement_archetype(request.usage),
            Some(request.usage),
            0,
        );
        if let Some(size) = ServiceBuildingSize::for_capacity(request.usage, request.capacity) {
            program = program.with_service_size(size);
        }
        let footprint = program.plot_dimensions_metres();
        let mut choices = candidates
            .iter()
            .filter(|candidate| !candidate.rear_court)
            .map(|candidate| service_candidate(*candidate, request, footprint))
            .filter(|candidate| inside_block(*candidate, blocks[&candidate.block_key]))
            .collect::<Vec<_>>();
        choices.sort_by_key(|candidate| {
            let distance = candidate.lot.centre_metres.length();
            let target = match district {
                BuildingDistrict::Market => 0.0,
                BuildingDistrict::Edge => radius,
                BuildingDistrict::Neighbourhood | BuildingDistrict::Craft => {
                    radius * (key as u16 as f32 / u16::MAX as f32)
                }
            };
            let band = ((distance - target).abs() / SERVICE_SPREAD_METRES) as u32;
            (band, mix64(key ^ candidate.selection_key))
        });
        if let Some(candidate) = choices.into_iter().find(|candidate| {
            placed
                .iter()
                .all(|other| !lots_overlap(candidate.lot, other.lot))
        }) {
            placed.push(candidate);
        } else {
            unplaced.push(request);
        }
    }
    (placed, unplaced)
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
    let half = candidate.lot.dimensions_metres() * 0.5;
    [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ]
    .into_iter()
    .all(|local| {
        let point = candidate.lot.centre_metres + candidate.lot.orientation.local_to_world(local);
        (0..4).all(|edge| {
            let start = block.corners[edge];
            let tangent = (block.corners[(edge + 1) % 4] - start).normalize();
            tangent.perp_dot(point - start) >= SERVICE_EDGE_CLEARANCE_METRES
        })
    })
}
