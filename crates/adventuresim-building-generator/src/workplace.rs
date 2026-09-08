//! Working buildings share structural components and reserve their entire working plot.
use adventuresim_world_schema::settlement_buildings::{BuildingUse, ServiceCapacity};
use bevy::math::{Vec2, Vec3};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{BuildingProgram, ResolvedItemId, WallAssemblyId};

mod assembly;
mod envelope;
mod equipment;
mod programme;
mod validation;
pub(crate) use assembly::resolve_workplace;
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
}

impl WorkplaceKind {
    pub const ALL: [Self; 6] = [
        Self::Barn,
        Self::Stable,
        Self::Granary,
        Self::Smithy,
        Self::Bakehouse,
        Self::MarketHall,
    ];
    pub const fn from_use(usage: BuildingUse) -> Option<Self> {
        match usage {
            BuildingUse::Barn => Some(Self::Barn),
            BuildingUse::Stable => Some(Self::Stable),
            BuildingUse::Granary => Some(Self::Granary),
            BuildingUse::Smithy | BuildingUse::Weaponsmith => Some(Self::Smithy),
            BuildingUse::Bakehouse => Some(Self::Bakehouse),
            BuildingUse::MarketHall => Some(Self::MarketHall),
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
        }
    }
}

/// Capacity bands select larger working footprints without multiplying every recipe per person.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum WorkplaceSize {
    Small,
    Medium,
    Large,
}

impl WorkplaceSize {
    pub fn for_capacity(usage: BuildingUse, capacity: ServiceCapacity) -> Option<Self> {
        WorkplaceKind::from_use(usage)?;
        let range = usage.definition().capacity;
        let span = range.maximum.0 - range.minimum.0 + 1;
        let band = capacity.0.saturating_sub(range.minimum.0).saturating_mul(3) / span;
        Some(match band {
            0 => Self::Small,
            1 => Self::Medium,
            _ => Self::Large,
        })
    }
    pub(crate) const fn extra_bays(self) -> u16 {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorkplaceMaterial {
    Timber,
    Masonry,
    Iron,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorkplaceFeature {
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
    pub size: WorkplaceSize,
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
            WorkplaceKind::Barn | WorkplaceKind::Stable | WorkplaceKind::MarketHall => {
                WorkplaceMaterial::Timber
            }
            _ => WorkplaceMaterial::Masonry,
        }
    }
}
