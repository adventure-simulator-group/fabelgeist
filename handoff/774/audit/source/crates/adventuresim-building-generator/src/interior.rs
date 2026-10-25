//! Rules-based furnishing with metre-space envelopes and front-door access proofs.
use crate::Direction;
use crate::furniture::{FurnitureAccessFace, FurnitureKey, FurnitureKind};
#[cfg(test)]
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod adversarial_tests;
mod architecture;
#[cfg(test)]
mod arrangement_tests;
mod budgets;
mod church;
mod composition;
mod exterior_access;
mod finishes;
mod footprints;
pub use exterior_access::StandingClearance;
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
pub use placement::{furnish, validate_layout};

/// Verify the completed architectural circulation before accepting a heated recipe.
pub fn validate_circulation(plan: &crate::BuildingPlan) -> InteriorResult<()> {
    navigation::Navigation::new(plan).map(|_| ())
}

/// Furniture front is local -Z. South therefore has zero renderer yaw.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InteriorPlacement {
    pub key: FurnitureKey,
    pub room_id: crate::RoomIndex,
    pub storey: crate::StoreyIndex,
    pub centre_metres: crate::plan_geometry::ArchitecturalPlanPoint,
    pub facing: Direction,
}
impl InteriorPlacement {
    pub fn yaw_radians(&self) -> crate::spatial_geometry::Radians {
        match self.facing {
            Direction::South => crate::spatial_geometry::Radians::ZERO,
            Direction::East => crate::spatial_geometry::Radians::NEGATIVE_QUARTER_TURN,
            Direction::North => crate::spatial_geometry::Radians::HALF_TURN,
            Direction::West => crate::spatial_geometry::Radians::QUARTER_TURN,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct InteriorWaypoint {
    pub storey: crate::StoreyIndex,
    pub position_metres: crate::plan_geometry::ArchitecturalPlanPoint,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FurnitureAccessPath {
    pub placement_index: usize,
    pub face: FurnitureAccessFace,
    pub points: Vec<InteriorWaypoint>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnmetFurnitureBudget {
    pub storey: crate::StoreyIndex,
    pub room_id: crate::RoomIndex,
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
/// Interior planning operations retain their layout error classification.
pub type InteriorResult<T> = std::result::Result<T, InteriorLayoutError>;

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum InteriorLayoutError {
    #[error(transparent)]
    StoreyElevation(#[from] crate::StoreyElevationError),
    #[error(transparent)]
    Ordinal(#[from] crate::OrdinalError),
    #[error("room {room} on storey {storey} is absent")]
    MissingRoom {
        storey: crate::StoreyIndex,
        room: crate::RoomIndex,
    },
    #[error("room {room} on storey {storey} has no required cell geometry")]
    EmptyRoomGeometry {
        storey: crate::StoreyIndex,
        room: crate::RoomIndex,
    },
    #[error("navigation floor on storey {storey} is absent")]
    MissingStoreyFloor { storey: crate::StoreyIndex },
    #[error("furniture {placement_index} has a broken access path at node {node}")]
    BrokenAccessPath { placement_index: usize, node: usize },
    #[error("furniture in room {room} on storey {storey} has no physical supporting floor")]
    MissingFloor {
        storey: crate::StoreyIndex,
        room: crate::RoomIndex,
    },
    #[error(transparent)]
    Furniture(#[from] crate::furniture::FurnitureRecipeError),
    #[error("invalid architectural collision: {0}")]
    Collision(#[from] crate::CollisionError),
    #[error("invalid interior geometry: {0}")]
    Geometry(#[from] crate::spatial_geometry::GeometryError),
    #[error("building has no accessible ground-floor front door")]
    MissingFrontDoor,
    #[error("room {room_id} on storey {storey} is disconnected from the front door")]
    DisconnectedRoom {
        storey: crate::StoreyIndex,
        room_id: crate::RoomIndex,
    },
    #[error("stair {index} has an inaccessible landing")]
    InvalidStair { index: usize },
    #[error("furniture {index} intersects architecture, another object, or a reserved doorway")]
    InvalidPlacement { index: usize },
    #[error("furniture {index} has an inaccessible usable face")]
    InaccessibleFurniture { index: usize },
    #[error("occupied building has no room for its primary furniture")]
    EmptyLayout,
}
