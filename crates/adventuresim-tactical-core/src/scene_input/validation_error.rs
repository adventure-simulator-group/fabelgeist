//! Stable scene admission failures and their exact owning causes.
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
pub enum SceneValidationError {
    #[error("imported source identity must be a lowercase SHA-256 digest")]
    SourceDigest,
    #[error("home catalog: {0}")]
    HomeCatalog(#[source] adventuresim_core::settlement_property::PropertyError),
    #[error("geographic heightmap is invalid")]
    GeographicHeightmap,
    #[error("geographic support source is invalid")]
    GeographicSupport,
    #[error("empty scene must not carry property support")]
    UnexpectedSupport,
    #[error("occupied scene lacks accepted property support")]
    MissingSupport,
    #[error("grounding requires the exact unbound producer layout")]
    ProducerLayout,
    #[error("registered home {home:?} and physical building {building} disagree at {location:?}")]
    HomeBinding {
        home: adventuresim_core::settlement_property::PropertyId,
        building: super::SceneBuildingId,
        location: Option<crate::scene_coordinates::ScenePlanPoint>,
    },
    #[error("registered home {home:?} is missing physical building {building}")]
    MissingHome {
        home: adventuresim_core::settlement_property::PropertyId,
        building: super::SceneBuildingId,
    },
    #[error("an imported settlement needs its physical home catalog")]
    MissingHomeCatalog,
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
    GardenOwnership { owner: SceneOwnerContext },
    #[error("garden owner must reference an occupied front building")]
    GardenOwner { owner: SceneOwnerContext },
    #[error("garden owner geometry lies outside its property")]
    GardenOwnerGeometry { owner: SceneOwnerContext },
    #[error("garden overlaps another owned property")]
    GardenOverlap { owner: SceneOwnerContext },
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
    CompoundIdentity { owner: SceneOwnerContext },
    #[error("compound front building is missing")]
    MissingCompoundFront { owner: SceneOwnerContext },
    #[error("compound rear building is missing")]
    MissingCompoundRear { owner: SceneOwnerContext },
    #[error("compound members have invalid roles or split simulation authority")]
    CompoundMemberAuthority { owner: SceneOwnerContext },
    #[error("compound plot, court or route count is invalid")]
    CompoundPlot { owner: SceneOwnerContext },
    #[error("compound access route is invalid")]
    CompoundAccess { owner: SceneOwnerContext },
    #[error("compound boundary wall is invalid")]
    CompoundWall { owner: SceneOwnerContext },
    #[error("compound gate is invalid")]
    CompoundGate { owner: SceneOwnerContext },
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
    BuildingOverlap {
        building: super::SceneBuildingId,
        other: super::SceneBuildingId,
    },
    #[error("building {building} program is invalid: {source}")]
    BuildingProgram {
        building: super::SceneBuildingId,
        #[source]
        source: adventuresim_building_generator::GenerationError,
    },
    #[error("building {building} interior: {source}")]
    BuildingInterior {
        building: super::SceneBuildingId,
        #[source]
        source: adventuresim_building_generator::interior::InteriorLayoutError,
    },
    #[error("distant furniture site {building}: {source}")]
    FurnitureSite {
        building: super::SceneBuildingId,
        #[source]
        source: adventuresim_building_generator::GenerationError,
    },
    #[error("city geometry: {0}")]
    City(#[source] crate::city_layout::CityCompileError),
    #[error("garden {owner:?}: {issue:?}")]
    Garden {
        owner: crate::city_layout::CityPropertyId,
        issue: crate::city_layout::gardens::GardenIssue,
    },
    #[error("garden clearance: {0}")]
    GardenClearance(#[source] crate::city_layout::GardenClearanceError),
    #[error("terrain recipe: {0}")]
    Terrain(#[source] crate::volumetric_terrain::TerrainRecipeError),
}

/// Rejected input may contain an invalid or repeated identity. Diagnostics keep
/// that exact attempted membership rather than admitting or normalizing it.
#[derive(Clone, Debug)]
pub struct SceneOwnerContext {
    pub property: crate::city_layout::CityPropertyId,
    pub members: Vec<super::SceneBuildingId>,
    pub location: Option<crate::scene_coordinates::ScenePlanPoint>,
}
impl SceneOwnerContext {
    pub(super) fn compound(compound: &crate::city_layout::CityCompound) -> Self {
        Self {
            property: compound.id,
            members: vec![compound.front_building_id, compound.rear_building_id],
            location: crate::scene_coordinates::ScenePlanPoint::from_metres(
                compound.plot.centre_metres(),
            ),
        }
    }
    pub(super) fn garden(garden: &crate::city_layout::CityGarden) -> Self {
        Self {
            property: garden.owner,
            members: vec![garden.front_building_id],
            location: crate::scene_coordinates::ScenePlanPoint::from_metres(
                garden.plot.centre_metres(),
            ),
        }
    }
}
