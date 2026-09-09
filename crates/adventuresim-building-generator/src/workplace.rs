//! Working buildings share structural components and reserve their entire working plot.
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::math::{Vec2, Vec3};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{BuildingProgram, ResolvedItemId, ServiceBuildingSize, WallAssemblyId};

mod assembly;
mod brewing;
mod craft;
mod envelope;
mod equipment;
mod horse_mill;
mod programme;
mod surfaces;
mod validation;
mod warehouse;
mod wet;
pub(crate) use assembly::resolve_workplace;
pub use surfaces::WorkplaceSurface;
pub(crate) use validation::audit_workplace;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum WorkplaceKind {
    Barn,
    Stable,
    Granary,
    Smithy,
    Bakehouse,
    MarketHall,
    Brewery,
    Malthouse,
    TimberYard,
    Carpenter,
    Warehouse,
    Dyer,
    Tannery,
    HorseMill,
}

impl WorkplaceKind {
    pub const ALL: [Self; 14] = [
        Self::Barn,
        Self::Stable,
        Self::Granary,
        Self::Smithy,
        Self::Bakehouse,
        Self::MarketHall,
        Self::Brewery,
        Self::Malthouse,
        Self::TimberYard,
        Self::Carpenter,
        Self::Warehouse,
        Self::Dyer,
        Self::Tannery,
        Self::HorseMill,
    ];
    pub const fn from_use(usage: BuildingUse) -> Option<Self> {
        match usage {
            BuildingUse::Barn => Some(Self::Barn),
            BuildingUse::Stable => Some(Self::Stable),
            BuildingUse::Granary => Some(Self::Granary),
            BuildingUse::Smithy | BuildingUse::Weaponsmith | BuildingUse::Armorer => {
                Some(Self::Smithy)
            }
            BuildingUse::Bakehouse => Some(Self::Bakehouse),
            BuildingUse::MarketHall => Some(Self::MarketHall),
            BuildingUse::Brewery => Some(Self::Brewery),
            BuildingUse::Malthouse => Some(Self::Malthouse),
            BuildingUse::TimberYard => Some(Self::TimberYard),
            BuildingUse::Carpenter => Some(Self::Carpenter),
            BuildingUse::Warehouse => Some(Self::Warehouse),
            BuildingUse::Dyer => Some(Self::Dyer),
            BuildingUse::Tannery => Some(Self::Tannery),
            BuildingUse::HorseMill => Some(Self::HorseMill),
            _ => None,
        }
    }
    pub const fn usage(self) -> BuildingUse {
        match self {
            Self::Barn => BuildingUse::Barn,
            Self::Stable => BuildingUse::Stable,
            Self::Granary => BuildingUse::Granary,
            Self::Smithy => BuildingUse::Smithy,
            Self::Bakehouse => BuildingUse::Bakehouse,
            Self::MarketHall => BuildingUse::MarketHall,
            Self::Brewery => BuildingUse::Brewery,
            Self::Malthouse => BuildingUse::Malthouse,
            Self::TimberYard => BuildingUse::TimberYard,
            Self::Carpenter => BuildingUse::Carpenter,
            Self::Warehouse => BuildingUse::Warehouse,
            Self::Dyer => BuildingUse::Dyer,
            Self::Tannery => BuildingUse::Tannery,
            Self::HorseMill => BuildingUse::HorseMill,
        }
    }
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Barn => "barn",
            Self::Stable => "stable",
            Self::Granary => "granary",
            Self::Smithy => "smithy",
            Self::Bakehouse => "bakehouse",
            Self::MarketHall => "market-hall",
            Self::Brewery => "brewery",
            Self::Malthouse => "malthouse",
            Self::TimberYard => "timber-yard",
            Self::Carpenter => "carpenter",
            Self::Warehouse => "warehouse",
            Self::Dyer => "dyer",
            Self::Tannery => "tannery",
            Self::HorseMill => "horse-mill",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorkplaceMaterial {
    Timber,
    /// Unfinished working wood stays independent from the building's painted facade palette.
    UnpaintedTimber,
    Grain,
    DyedCloth,
    UndyedCloth,
    Hide,
    ProcessLiquid,
    HempRope,
    Masonry,
    DressedStone,
    Iron,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorkplaceFeature {
    TimberStack,
    SawBench,
    Wall,
    Post,
    Beam,
    Boarding,
    Floor,
    Stair,
    Stall,
    Trough,
    StorageBin,
    Hoist,
    Forge,
    Anvil,
    Oven,
    Flue,
    Counter,
    Fence,
    Rack,
    Vat,
    Kiln,
    Louver,
    LoadingHoist,
    MillDrive,
    MillSweep,
    Millstone,
    Hopper,
    DyeKettle,
    SoakingTank,
    DryingFrame,
    Cloth,
    Hide,
    FleshingBeam,
    /// Rendered liquid fill; the vessel's rim and bottom supply physical collision.
    ProcessLiquid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkplacePart {
    pub solid: ResolvedItemId,
    pub feature: WorkplaceFeature,
    pub material: WorkplaceMaterial,
    /// Large architectural components remain present in distant representations.
    pub silhouette: bool,
}

/// Clear space is a geometric contract, including the open passage between independent bays.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkplacePassage {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkplacePlan {
    pub kind: WorkplaceKind,
    pub size: ServiceBuildingSize,
    pub plot_dimensions_metres: Vec2,
    pub walls: Vec<WallAssemblyId>,
    pub parts: Vec<WorkplacePart>,
    pub passages: Vec<WorkplacePassage>,
}

impl BuildingProgram {
    pub fn workplace_kind(&self) -> Option<WorkplaceKind> {
        self.usage.and_then(WorkplaceKind::from_use)
    }
    pub fn plot_dimensions_metres(&self) -> Vec2 {
        let (width, depth) = self.footprint.dimensions();
        let main = Vec2::new(f32::from(width), f32::from(depth)) * crate::CELL_SIZE_METRES;
        main + self
            .workplace_kind()
            .map_or(Vec2::ZERO, |kind| Vec2::new(kind.yard_width_metres(), 0.0))
    }
}

#[cfg(test)]
mod tests;

impl WorkplacePlan {
    pub(crate) fn gable_material(&self) -> WorkplaceMaterial {
        match self.kind {
            WorkplaceKind::Barn
            | WorkplaceKind::Stable
            | WorkplaceKind::MarketHall
            | WorkplaceKind::Brewery
            | WorkplaceKind::TimberYard
            | WorkplaceKind::Carpenter
            | WorkplaceKind::Warehouse
            | WorkplaceKind::Dyer
            | WorkplaceKind::Tannery
            | WorkplaceKind::HorseMill => WorkplaceMaterial::Timber,
            _ => WorkplaceMaterial::Masonry,
        }
    }
}
