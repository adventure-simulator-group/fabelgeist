//! Deterministic urban plots around a surveyed street graph and staggered old-quarter lanes.

use adventuresim_core::settlement_property::{HomeSupplyRole, ResidentCount};
use std::collections::BTreeSet;

use crate::scene_coordinates::{PlanDisplacement, ScenePlanPoint};
use adventuresim_building_generator::spatial_geometry::{
    GeometryError, GeometryResult, PlanDimensions,
};
use bevy::math::Vec2;
use fabelgeist_determinism::StreamId;

use crate::scene_input::BuildingOrientation;
use adventuresim_world_schema::{
    SettlementEconomyProfile,
    settlement_buildings::{
        BuildingDemand, BuildingUse, DemandShortfall, ParishProgramme, SettlementBuildingDemand,
    },
};
pub use compiled::{
    ChurchSitingIssue, CityBusinessSite, CityCompileError, CityCompileResult, CityGroundingError,
    CityGroundingProjection, CityGroundingProjectionError, CityRecipePalette, CitySceneLayout,
    CitySingleProperty, CitySupportError, CompiledCityLayout, CompoundGradingPolicy, CompoundIssue,
    GardenClearanceError, GroundedCitySceneLayout, ProjectionBoundary, ProjectionOwnerContext,
    SelectedCityGrounding, SinglePropertyGradingPolicy, StreetApronDimensions,
};
pub(crate) use compiled::{validate_scene_compound, validate_scene_gardens};
pub use compound::{
    BoundaryGeometryError, CityAccessSegment, CityBoundary, CityBoundaryMaterial,
    CityBoundaryMember, CityBoundaryPose, CityBoundarySegment, CityCompound, CityGate,
    CityPlotBounds, CityPropertyId, MAX_CITY_BUILDING_INSTANCES, PropertySide,
};
pub use gardens::{
    CityGarden, GardenPlantId, GardenPlantPlacement, GardenPlantScale, GardenSpecimen,
};
pub use graph::BlockId;
use graph::{CityBlock, StreetClass, StreetGraph};
pub use parishes::{CITY_PARISH_PRECINCT_RADIUS_METRES, CityParish, ParishResidenceAllocation};
#[cfg(test)]
use plots::lots_overlap;
pub use site::CitySite;
use site::DevelopmentExtent;

pub use houses::CityHouseClass;
pub use packing::{
    CityPackingIssue, ExploredSearchNodes, FrontageDisplacement, FrontageInterval,
    FrontageIntervalError, SearchBudget,
};
pub use surfaces::{
    CityStreetPatch, CityStreetSurface, CityYardPatch, CityYardSurface, MAX_CITY_STREET_PATCHES,
    MAX_CITY_YARD_PATCHES,
};
use surfaces::{city_street_patches, city_yard_patches};

mod compiled;
mod compound;
pub mod gardens;
mod graph;
pub mod grounding;
mod houses;
mod packing;
mod parishes;
mod plots;
mod residences;
mod services;
mod site;
mod subdivision;
mod surfaces;
const RNG_CITY_PASSAGE_SIDE: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("city.passage-side");

const CITY_RADIUS_X_METRES: f32 = 1_300.0;
const CITY_RADIUS_Y_METRES: f32 = 1_260.0;
const ORDINARY_STREET_HALF_WIDTH_METRES: f32 = 2.0;
const SECONDARY_STREET_HALF_WIDTH_METRES: f32 = 3.0;
const PRIMARY_STREET_HALF_WIDTH_METRES: f32 = 4.0;
const FRONTAGE_CORNER_CLEARANCE_METRES: f32 = 4.0;
const PARTY_WALL_CLEARANCE_METRES: f32 = 0.12;
pub(crate) const SPATIAL_BUCKET_METRES: f32 = 32.0;
pub const MAX_CITY_LOTS: usize = 16_384;

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedCityLayout {
    pub prosperity: adventuresim_world_schema::ProsperityTier,
    pub lots: Vec<CityBuildingLot>,
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
    pub unplaced_services: Vec<BuildingDemand>,
    pub demand_shortfalls: Vec<DemandShortfall>,
    pub parishes: Vec<ParishProgramme>,
    pub unhoused_population: ResidentCount,
    packing: CityCompileResult<packing::CityPackingContext>,
}

/// One rectangular building lot aligned to one locally straight street frontage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityBuildingLot {
    pub passage_side: PropertySide,
    pub id: CityPropertyId,
    pub centre_metres: ScenePlanPoint,
    pub orientation: BuildingOrientation,
    pub house_class: CityHouseClass,
    pub footprint_metres: PlanDimensions,
    pub service: Option<BuildingDemand>,
}

#[derive(Clone, Copy)]
struct CandidateLot {
    lot: CityBuildingLot,
    block_key: BlockId,
    position: LotPosition,
    selection_key: u64,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, thiserror::Error)]
pub enum CityPlanningIssue {
    #[error("the street graph has no surveyed market")]
    MissingMarket,
    #[error("a developed frontage is disconnected from the trade route")]
    DisconnectedFrontage,
    #[error("street patches require {required} instances, exceeding {maximum}")]
    StreetPatchLimit { required: usize, maximum: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum LotPosition {
    StreetFrontage,
    RearCourt,
}

impl CityBuildingLot {
    pub fn front_building_id(self) -> crate::scene_input::SceneBuildingId {
        crate::scene_input::SceneBuildingId(self.id.0)
    }
    pub fn bounds(self) -> GeometryResult<CityPlotBounds> {
        CityPlotBounds::new(self.centre_metres, self.footprint_metres, self.orientation)
    }
}

/// Reserves service frontage, then houses residents around the same connected street graph.
impl CitySite {
    pub fn generate(
        &self,
        seed: fabelgeist_determinism::Seed,
        resident_population: ResidentCount,
        economy: &SettlementEconomyProfile,
    ) -> CityCompileResult<GeneratedCityLayout> {
        let extent = DevelopmentExtent::for_population(resident_population)?;
        let graph = self.street_graph(seed, extent)?;
        let mut candidates = graph
            .blocks
            .iter()
            .copied()
            .filter(|block| block_is_inside_city(*block, extent) && !block.is_market())
            .map(|block| block_lots(seed, block))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| {
            (
                candidate.position,
                candidate.selection_key,
                candidate.block_key,
            )
        });
        let demand = SettlementBuildingDemand::with_parish_policy(
            seed,
            resident_population.get(),
            economy,
            self.parish_policy,
        );
        if !demand.shortfalls.is_empty() {
            return Ok(GeneratedCityLayout {
                prosperity: economy.prosperity_tier,
                lots: Vec::new(),
                streets: Vec::new(),
                yards: Vec::new(),
                unplaced_services: demand.buildings,
                demand_shortfalls: demand.shortfalls,
                parishes: demand.parishes,
                unhoused_population: resident_population,
                packing: Ok(packing::CityPackingContext::default()),
            });
        }

        let services::ServicePlacement {
            placed: service_lots,
            unplaced: unplaced_services,
        } = services::place_services(
            seed,
            resident_population,
            &graph.blocks,
            &candidates,
            &demand.buildings,
        )?;
        let mut candidates = residences::pack(seed, &graph.blocks, extent, service_lots)?;
        let development_order = graph.development_order();
        candidates.sort_by_key(|candidate| {
            (
                candidate.lot.service.is_none(),
                candidate.position,
                development_order[&candidate.block_key],
                candidate.selection_key,
            )
        });

        let roster = residences::SelectedRoster::from_candidates(candidates, resident_population);
        let selected = roster.members;
        let developed_blocks = selected
            .iter()
            .map(|candidate| candidate.block_key)
            .collect::<BTreeSet<_>>();
        let packing = packing::CityPackingContext::from_selected(&selected, &graph.blocks);
        let yards = city_yard_patches(&selected)?;
        let streets = city_street_patches(&graph, &developed_blocks)?;
        Ok(GeneratedCityLayout {
            prosperity: economy.prosperity_tier,
            lots: selected
                .into_iter()
                .map(|candidate| candidate.lot)
                .collect(),
            streets,
            yards,
            unplaced_services,
            demand_shortfalls: demand.shortfalls,
            parishes: demand.parishes,
            unhoused_population: roster.unhoused_population,
            packing,
        })
    }
}

fn block_is_inside_city(block: CityBlock, extent: DevelopmentExtent) -> bool {
    let centre = block.centre();
    (centre.x / CITY_RADIUS_X_METRES).powi(2) + (centre.y / extent.radius_y_metres.metres()).powi(2)
        <= 1.0
}

fn block_lots(
    seed: fabelgeist_determinism::Seed,
    block: CityBlock,
) -> GeometryResult<Vec<CandidateLot>> {
    let mut lots = Vec::new();
    for edge_index in 0..4 {
        append_frontage(
            &mut lots,
            seed,
            StreamId::new("city.frontage-identity")
                .seed(block.key().0.into(), &[edge_index as u64]),
            block.key(),
            block.corners[edge_index],
            block.corners[(edge_index + 1) % 4],
            block.streets[edge_index].half_width()?,
        )?;
    }
    let mut accepted = Vec::new();
    for candidate in lots {
        if plots::inside_block(candidate.lot, block)? {
            accepted.push(candidate);
        }
    }
    Ok(accepted)
}

fn append_frontage(
    lots: &mut Vec<CandidateLot>,
    seed: fabelgeist_determinism::Seed,
    run_key: fabelgeist_determinism::Seed,
    block_key: BlockId,
    start: ScenePlanPoint,
    end: ScenePlanPoint,
    street_half_width: adventuresim_building_generator::spatial_geometry::PositiveLength,
) -> GeometryResult<()> {
    // The frontage arithmetic kernel preserves the existing f32 operation order.
    let start = start.metres();
    let end = end.metres();
    let street_half_width = street_half_width.metres();
    let displacement = end - start;
    let length = displacement.length();
    let tangent = displacement / length;
    let inward = Vec2::new(-tangent.y, tangent.x);
    let mut cursor = FRONTAGE_CORNER_CLEARANCE_METRES;
    let mut index = 0_u64;
    loop {
        let lot_key = CityPropertyId(
            StreamId::new("city.lot-identity")
                .seed(run_key, &[index])
                .to_u64(),
        );
        let house_class = house_class(seed, lot_key, LotPosition::StreetFrontage);
        let compound_margin = if house_class == CityHouseClass::MerchantHouse {
            compound::COMPOUND_EDGE_MARGIN_METRES
        } else {
            0.0
        };
        let footprint = house_class.footprint()?;
        let frontage = footprint.metres().x + compound_margin * 2.0;
        if cursor + frontage > length - FRONTAGE_CORNER_CLEARANCE_METRES {
            break;
        }
        // Keep the reserved parcel fixed when its side passage is mirrored.
        let passage_offset = match passage_side(seed, lot_key, house_class) {
            PropertySide::Left => plots::SIDE_PASSAGE_METRES,
            PropertySide::Right => 0.0,
        };
        let street_point = start + tangent * (cursor + frontage * 0.5 + passage_offset);
        lots.push(candidate(
            seed,
            lot_key,
            block_key,
            ScenePlanPoint::try_from(
                street_point
                    + inward * (street_half_width + compound_margin + footprint.metres().y * 0.5),
            )?,
            tangent,
            house_class,
            LotPosition::StreetFrontage,
        )?);
        cursor += frontage + plots::SIDE_PASSAGE_METRES + PARTY_WALL_CLEARANCE_METRES;
        index += 1;
    }
    Ok(())
}

fn passage_side(
    seed: fabelgeist_determinism::Seed,
    lot_key: CityPropertyId,
    house_class: CityHouseClass,
) -> PropertySide {
    if house_class == CityHouseClass::MerchantHouse
        && RNG_CITY_PASSAGE_SIDE.rng(seed, &[lot_key.0]).boolean()
    {
        PropertySide::Left
    } else {
        PropertySide::Right
    }
}

fn candidate(
    seed: fabelgeist_determinism::Seed,
    lot_key: CityPropertyId,
    block_key: BlockId,
    centre_metres: ScenePlanPoint,
    frontage_tangent: Vec2,
    house_class: CityHouseClass,
    position: LotPosition,
) -> GeometryResult<CandidateLot> {
    Ok(CandidateLot {
        lot: CityBuildingLot {
            passage_side: passage_side(seed, lot_key, house_class),
            id: lot_key,
            centre_metres,
            orientation: BuildingOrientation::from_frontage_tangent(frontage_tangent)
                .ok_or(GeometryError::InvalidProjection)?,
            house_class,
            footprint_metres: house_class.footprint()?,
            service: None,
        },
        block_key,
        position,
        selection_key: StreamId::new("city.development-priority")
            .rng(seed, &[lot_key.0])
            .next_u64(),
    })
}

fn house_class(
    seed: fabelgeist_determinism::Seed,
    lot_key: CityPropertyId,
    position: LotPosition,
) -> CityHouseClass {
    let mut random = StreamId::new("city.house-class").rng(seed, &[lot_key.0]);
    if position == LotPosition::RearCourt {
        return if random.index(5) == 0 {
            CityHouseClass::CraftTownHouse
        } else {
            CityHouseClass::Cottage
        };
    }
    match random.index(16) {
        0..=4 => CityHouseClass::Cottage,
        5..=11 => CityHouseClass::CraftTownHouse,
        12..=13 => CityHouseClass::HallHouse,
        _ => CityHouseClass::MerchantHouse,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod capacity_tests;
