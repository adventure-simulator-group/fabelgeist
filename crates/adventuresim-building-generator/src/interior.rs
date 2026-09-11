//! Rules-based furnishing with metre-space envelopes and front-door access proofs.
use crate::Direction;
use crate::furniture::{FurnitureAccessFace, FurnitureKey, FurnitureKind};
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod adversarial_tests;
mod architecture;
#[cfg(test)]
mod arrangement_tests;
mod budgets;
mod composition;
mod geometry;
mod navigation;
mod obstruction;
mod placement;
mod room_facing;
#[cfg(test)]
mod spiral_tests;
#[cfg(test)]
mod tests;
pub use budgets::{FurnitureBudget, FurniturePosition, furniture_budgets};
pub use geometry::furniture_floor_height;
pub use placement::{furnish, validate_layout};

/// Furniture front is local -Z. South therefore has zero renderer yaw.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InteriorPlacement {
    pub key: FurnitureKey,
    pub room_id: u16,
    pub storey: u16,
    pub centre_metres: Vec2,
    pub facing: Direction,
}
impl InteriorPlacement {
    pub fn yaw_radians(&self) -> f32 {
        match self.facing {
            Direction::South => 0.0,
            Direction::East => -std::f32::consts::FRAC_PI_2,
            Direction::North => std::f32::consts::PI,
            Direction::West => std::f32::consts::FRAC_PI_2,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct InteriorWaypoint {
    pub storey: u16,
    pub position_metres: Vec2,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FurnitureAccessPath {
    pub placement_index: usize,
    pub face: FurnitureAccessFace,
    pub points: Vec<InteriorWaypoint>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnmetFurnitureBudget {
    pub storey: u16,
    pub room_id: u16,
    pub kind: FurnitureKind,
    pub requested: usize,
    pub placed: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InteriorLayout {
    pub placements: Vec<InteriorPlacement>,
    pub paths: Vec<FurnitureAccessPath>,
    pub unmet_budgets: Vec<UnmetFurnitureBudget>,
}
#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub enum InteriorLayoutError {
    #[error("building has no accessible ground-floor front door")]
    MissingFrontDoor,
    #[error("room {room_id} on storey {storey} is disconnected from the front door")]
    DisconnectedRoom { storey: u16, room_id: u16 },
    #[error("stair {index} has an inaccessible landing")]
    InvalidStair { index: usize },
    #[error("furniture {index} intersects architecture, another object, or a reserved doorway")]
    InvalidPlacement { index: usize },
    #[error("furniture {index} has an inaccessible usable face")]
    InaccessibleFurniture { index: usize },
    #[error("occupied building has no room for its primary furniture")]
    EmptyLayout,
}
