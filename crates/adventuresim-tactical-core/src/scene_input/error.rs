//! Structured scene contract failures with retained generation causes.
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleGridKind {
    Playable,
    Vista,
}
impl std::fmt::Display for SampleGridKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Playable => "playable",
            Self::Vista => "vista",
        })
    }
}

#[derive(Debug, Error)]
pub enum SceneInputError {
    #[error("scene input I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("scene input JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("scene input is invalid: {0}")]
    Validation(#[from] SceneValidationError),
}

#[derive(Debug, Error)]
pub enum SceneValidationError {
    #[error("garden requires a level terrace across its complete vista footprint and LOD morph")]
    GardenLevelTerrace,
    #[error("incompatible schema version")]
    SchemaVersion,
    #[error("incompatible generation version")]
    GenerationVersion,
    #[error("scene key is empty or oversized")]
    SceneKey,
    #[error("source identity is empty or oversized")]
    SourceIdentity,
    #[error("geographic origin is out of bounds")]
    GeographicOrigin,
    #[error("vista has too many LOD levels")]
    VistaLevelCount,
    #[error("vista LOD levels are not strictly increasing")]
    VistaLevelOrder,
    #[error("vista LOD origin is not finite")]
    VistaOrigin,
    #[error("vista LOD spacing must progressively increase")]
    VistaSpacingOrder,
    #[error("vista sample count exceeds its bound")]
    VistaSampleCount,
    #[error("weather snapshot is invalid")]
    Weather,
    #[error("scene has too many tactical buildings")]
    BuildingCount,
    #[error("building identity is zero or duplicated")]
    BuildingIdentity,
    #[error("building placement is invalid")]
    BuildingPlacement,
    #[error("scene has too many distant buildings")]
    DistantBuildingCount,
    #[error("distant building identity is zero or duplicated")]
    DistantBuildingIdentity,
    #[error("distant building placement is invalid")]
    DistantBuildingPlacement,
    #[error("scene exceeds garden count bound")]
    GardenCount,
    #[error("garden ownership, membership or plant identity is invalid")]
    GardenOwnership,
    #[error("garden owner must reference an occupied front building")]
    GardenOwner,
    #[error("garden owner geometry lies outside its property")]
    GardenOwnerGeometry,
    #[error("garden overlaps another owned property")]
    GardenOverlap,
    #[error("establishment identity or operator is empty")]
    EstablishmentIdentity,
    #[error("establishment building, business, or operator is duplicated")]
    EstablishmentDuplicate,
    #[error("establishments cross settlement boundaries")]
    EstablishmentSettlement,
    #[error("establishment references an unknown business building")]
    EstablishmentBuilding,
    #[error("establishment business use does not match its building")]
    EstablishmentUsage,
    #[error("establishment shop name does not match its operator and building use")]
    EstablishmentShopName,
    #[error("scene street surfaces are invalid or exceed their bound")]
    Streets,
    #[error("scene yard surfaces are invalid or exceed their bound")]
    Yards,
    #[error("distant garden owner does not share its graded vista elevation")]
    GardenVistaElevation,
    #[error("boundary garden requires graded vista terrain")]
    GardenVistaRequired,
    #[error("garden leaves the supplied vista domain")]
    GardenVistaDomain,
    #[error("garden terrace does not match the final stitched terrain")]
    GardenTerrace,
    #[error("scene exceeds parish count bound")]
    ParishCount,
    #[error("parish identity or population is invalid")]
    ParishIdentity,
    #[error("town parish programme has an invalid principal or school count")]
    ParishProgramme,
    #[error("settlement has a building without its parish association")]
    MissingParishAssociation,
    #[error("parish church is missing")]
    MissingParishChurch,
    #[error("parish church recipe disagrees with its programme")]
    ParishChurchProgramme,
    #[error("parish member is missing, shared, or has the wrong use")]
    ParishMember,
    #[error("parish support building is outside its precinct")]
    ParishPrecinct,
    #[error("town school must belong to the principal parish")]
    SchoolParish,
    #[error("parish residential allocation is missing, shared, or not housing")]
    ParishHousing,
    #[error("parish allocation exceeds physical housing capacity")]
    ParishHousingCapacity,
    #[error("parish population differs from its residential allocation")]
    ParishPopulation,
    #[error("scene exceeds compound count bound")]
    CompoundCount,
    #[error("building identity occurs in both simulation and distant presentation")]
    SplitBuildingAuthority,
    #[error("compound identity or membership is invalid")]
    CompoundIdentity,
    #[error("compound front building is missing")]
    MissingCompoundFront,
    #[error("compound rear building is missing")]
    MissingCompoundRear,
    #[error("compound members have invalid roles or split simulation authority")]
    CompoundMemberAuthority,
    #[error("compound plot, court or route count is invalid")]
    CompoundPlot,
    #[error("compound access route is invalid")]
    CompoundAccess,
    #[error("compound boundary wall is invalid")]
    CompoundWall,
    #[error("compound gate is invalid")]
    CompoundGate,
    #[error("connected property terraces require conflicting vista elevations")]
    PropertyTerraceConflict,
    #[error("file exceeds the 32 MiB bound")]
    FileSize,
    #[error("vista sample count overflow")]
    VistaSampleOverflow,
    #[error("authoritative detail terrain is invalid")]
    DetailTerrain,
    #[error("playable heightmap is invalid")]
    PlayableTerrain,
    #[error("generated ground-surface grid is invalid")]
    GroundSurface,
    #[error("{grid} dimensions are out of bounds")]
    GridDimensions { grid: SampleGridKind },
    #[error("{grid} spacing is out of bounds")]
    GridSpacing { grid: SampleGridKind },
    #[error("{grid} sample counts do not match dimensions")]
    GridSamples { grid: SampleGridKind },
    #[error("{grid} contains an invalid height")]
    GridHeight { grid: SampleGridKind },
    #[error("{grid} contains an invalid environment sample")]
    GridEnvironment { grid: SampleGridKind },
    #[error("{grid} dimensions overflow")]
    GridOverflow { grid: SampleGridKind },
    #[error("building {building} footprint overlaps building {other}")]
    BuildingOverlap { building: u64, other: u64 },
    #[error("building {building} program is invalid: {source}")]
    BuildingProgram {
        building: u64,
        #[source]
        source: adventuresim_building_generator::GenerationError,
    },
    #[error("building {building} interior: {source}")]
    BuildingInterior {
        building: u64,
        #[source]
        source: adventuresim_building_generator::interior::InteriorLayoutError,
    },
    #[error("distant furniture site {building}: {source}")]
    FurnitureSite {
        building: u64,
        #[source]
        source: adventuresim_building_generator::GenerationError,
    },
    #[error("city geometry: {0}")]
    City(#[source] crate::city_layout::CityCompileError),
    #[error("garden {owner:?}: {issue:?}")]
    Garden {
        owner: crate::city_layout::CityPropertyId,
        issue: crate::city_layout::GardenIssue,
    },
    #[error("garden clearance: {0}")]
    GardenClearance(#[source] crate::city_layout::GardenClearanceError),
    #[error("terrain recipe: {0}")]
    Terrain(#[source] crate::volumetric_terrain::TerrainRecipeError),
}
