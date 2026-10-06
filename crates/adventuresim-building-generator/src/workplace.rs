//! Working buildings share structural components and reserve their entire working plot.
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::math::{Vec2, Vec3};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::spatial_geometry::{Architectural, ClearanceVolume, PlanDimensions};

use crate::{BuildingProgram, ResolvedItemId, ServiceBuildingSize, WallAssemblyId};

#[cfg(test)]
mod admission_tests;
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
pub use assembly::{PartAuthority, WorkplaceConstructionError};
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
    Millstone,
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
    pub silhouette: WorkplacePartVisibility,
}

/// Whether a physical part remains in the distant architectural silhouette.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkplacePartVisibility {
    Silhouette,
    DetailOnly,
}
impl WorkplacePartVisibility {
    pub const fn contributes_to_silhouette(self) -> bool {
        matches!(self, Self::Silhouette)
    }
}
impl Serialize for WorkplacePartVisibility {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.contributes_to_silhouette().serialize(s)
    }
}
impl<'de> Deserialize<'de> for WorkplacePartVisibility {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(if bool::deserialize(d)? {
            Self::Silhouette
        } else {
            Self::DetailOnly
        })
    }
}

/// Stable identity of one authored clear passage in an occupied workplace.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkplacePassageId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkplacePassagePurpose {
    GroundFloorCirculation,
    OutdoorRoute,
    UpperCirculation,
    ServiceClearance,
}

/// Authored clearance volume with a stable identity and explicit use.
#[derive(Clone, Debug)]
pub struct WorkplacePassage {
    pub id: WorkplacePassageId,
    pub purpose: WorkplacePassagePurpose,
    pub bounds: ClearanceVolume<Architectural>,
}

impl Serialize for WorkplacePassage {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut value = serializer.serialize_struct("WorkplacePassage", 4)?;
        value.serialize_field("id", &self.id)?;
        value.serialize_field("purpose", &self.purpose)?;
        value.serialize_field("min", &self.bounds.min())?;
        value.serialize_field("max", &self.bounds.max())?;
        value.end()
    }
}
impl<'de> Deserialize<'de> for WorkplacePassage {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Passage {
            id: WorkplacePassageId,
            purpose: WorkplacePassagePurpose,
            min: Vec3,
            max: Vec3,
        }
        let value = Passage::deserialize(d)?;
        let bounds = crate::spatial_geometry::SpatialBounds::from_metres(value.min, value.max)
            .and_then(ClearanceVolume::new)
            .map_err(|cause| {
                serde::de::Error::custom(WorkplaceConstructionError::Passage {
                    id: value.id,
                    cause,
                })
            })?;
        Ok(Self {
            id: value.id,
            purpose: value.purpose,
            bounds,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkplacePlan {
    pub kind: WorkplaceKind,
    pub size: ServiceBuildingSize,
    pub plot_dimensions_metres: PlanDimensions,
    pub walls: Vec<WallAssemblyId>,
    pub parts: Vec<WorkplacePart>,
    pub passages: Vec<WorkplacePassage>,
}

impl BuildingProgram {
    pub fn workplace_kind(&self) -> Option<WorkplaceKind> {
        self.usage.and_then(WorkplaceKind::from_use)
    }
    pub fn plot_dimensions_metres(&self) -> Vec2 {
        if self.church_program.is_some() {
            return crate::church_programme::URBAN_BASILICA_PLOT_METRES;
        }
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
