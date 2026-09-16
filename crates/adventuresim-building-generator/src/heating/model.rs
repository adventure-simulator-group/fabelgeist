//! Programme, ownership and physical connections of one domestic heating core.

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    BuildingLodMaterial, GeometryOwnerId, ResolvedBounds, ResolvedItemId, RoofAssemblyId,
    StructuralNodeId, WallAssemblyId,
};

/// A selected construction programme, not a claim that every house had a chimney.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomesticHeatingProgramme {
    HearthAndRearFedStove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HeatingRoom {
    pub storey_level: u16,
    pub room_id: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatingPartKind {
    Footing,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomesticHeatingPlan {
    pub programme: DomesticHeatingProgramme,
    pub owner: GeometryOwnerId,
    pub kitchen: HeatingRoom,
    pub heated_room: HeatingRoom,
    pub fire_wall: WallAssemblyId,
    pub centre_metres: Vec2,
    pub kitchen_axis: Vec2,
    pub floor_height_metres: f32,
    pub ground_support: StructuralNodeId,
    pub parts: Vec<HeatingPart>,
    pub passages: Vec<HeatingPassage>,
    pub operating_space: ResolvedBounds,
    pub roof: HeatingRoofPenetration,
}
