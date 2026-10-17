//! Programme, ownership and physical connections of one domestic heating core.
use crate::{Architectural, SpatialBounds};

use serde::{Deserialize, Serialize};

use crate::{
    BuildingLodMaterial, GeometryOwnerId, ResolvedItemId, RoofAssemblyId, StructuralNodeId,
    WallAssemblyId,
};

/// A selected construction programme, not a claim that every house had a chimney.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomesticHeatingProgramme {
    HearthAndRearFedStove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HeatingRoom {
    pub storey_level: crate::StoreyIndex,
    pub room_id: crate::RoomIndex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatingPartKind {
    Footing,
    SupportPier,
    FloorClosure,
    FlueShoulder,
    Hearth,
    FireWall,
    TiledStove,
    Hood,
    Flue,
    RoofFlashing,
    RoofUpstand,
    RoofCounterFlashing,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HeatingPart {
    pub solid: ResolvedItemId,
    pub kind: HeatingPartKind,
    pub material: BuildingLodMaterial,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatingPassageKind {
    HearthMouth,
    StoveFirebox,
    StoveChamber,
    StoveSmokeReturn,
    HoodThroat,
    FlueBore,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HeatingPassage {
    pub kind: HeatingPassageKind,
    pub void: ResolvedItemId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeatingRoofPenetration {
    pub roof: RoofAssemblyId,
    pub face: ResolvedItemId,
    pub cutout_index: usize,
    pub edges: Vec<ResolvedItemId>,
    pub flashing: Vec<ResolvedItemId>,
}

/// A physical deck opening and its noncombustible, supported cover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeatingFloorPenetration {
    pub storey_level: crate::StoreyIndex,
    pub core: SpatialBounds<Architectural>,
    pub cut: SpatialBounds<Architectural>,
    pub closures: Vec<ResolvedItemId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomesticHeatingPlan {
    pub programme: DomesticHeatingProgramme,
    pub owner: GeometryOwnerId,
    pub kitchen: HeatingRoom,
    pub heated_room: HeatingRoom,
    pub fire_wall: WallAssemblyId,
    pub centre_metres: crate::plan_geometry::ArchitecturalPlanPoint,
    pub kitchen_axis: crate::spatial_geometry::PlanDirection<crate::Architectural>,
    pub floor_height_metres: crate::spatial_geometry::Elevation<crate::Architectural>,
    pub ground_support: StructuralNodeId,
    pub parts: Vec<HeatingPart>,
    pub passages: Vec<HeatingPassage>,
    pub operating_space: SpatialBounds<Architectural>,
    pub roof: HeatingRoofPenetration,
    pub floors: Vec<HeatingFloorPenetration>,
}
