//! Pure generation jobs shared by browser workers and the renderer.
//! Products contain immutable scene assets, never tactical simulation state.
use adventuresim_building_generator::BuildingProgram;
use adventuresim_tactical_core::scene_input::{GeneratedTacticalScene, TacticalSceneInput};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;

use crate::presentation::ownership::{PresentationOwner, PresentationOwners};
use adventuresim_tactical_core::geometry_transport::binary as packed;
pub(super) use building::PreparedFacade;
pub(in crate::presentation) use city::PreparedCityProduct;
use error::{PreparationError, PreparationResult, ProductKind};
pub(super) use installation::{release_unused_facades, take_facade, take_venue_geometry};
pub(crate) use ownership::PreparationTicket;
pub(crate) use ownership::activate;
pub(in crate::presentation) use ownership::activate_city;
use ownership::{active_products, begin, cancel, finish, staged_products};
pub(super) use ownership::{clear_residency, retain_facade};
use requests::{dependencies, jobs, venue_jobs};
use residency::SceneReadiness;
pub(super) use venue::VenueGeometry;
use worker_products::{generate, receive};

mod building;
mod city;
mod error;
mod installation;
pub(crate) mod landscape;
mod ownership;
mod requests;
mod residency;
mod scene;
pub(super) mod venue;
#[cfg(target_family = "wasm")]
mod wasm;
#[cfg(target_family = "wasm")]
mod wasm_error;
mod worker_products;

const RETAINED_VENUE_RECIPES: usize = 64;
const RETAINED_SCENE_PRODUCTS: usize = 3;

#[derive(Clone, Default)]
struct PreparedProducts {
    regional_city: Option<Arc<city::PreparedCityProduct>>,
    ground: Vec<landscape::GroundProduct>,
    grass: Vec<landscape::GrassProduct>,
    active_ground: Option<landscape::LandscapeIdentity>,
    scenes: Vec<Arc<GeneratedTacticalScene>>,
    facades: Vec<Arc<PreparedFacade>>,
    sites: Vec<adventuresim_tactical_core::scene_input::ProgramFurnitureSite>,
    venues: Vec<venue::PreparedVenue>,
    placements: Vec<adventuresim_tactical_core::scene_input::TacticalBuildingPlacement>,
}
#[derive(Default, Serialize, Deserialize)]
struct Dependencies {
    grass: Option<landscape::GrassDependencies>,
    ground: Option<landscape::GroundDependencies>,
    sites: Vec<adventuresim_tactical_core::scene_input::ProgramFurnitureSite>,
    recipe: Option<adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe>,
    playable: Vec<adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe>,
}

#[derive(Serialize, Deserialize)]
enum GenerationJob {
    RegionalCity {
        input: Box<adventuresim_tactical_core::regional_city::RegionalCityInput>,
        graphics: String,
    },
    Scene(Box<TacticalSceneInput>),
    Building(Box<BuildingProgram>),
    Venue(Box<BuildingProgram>),
    Grass {
        input: Box<TacticalSceneInput>,
        graphics: String,
    },
    Ground {
        input: Box<TacticalSceneInput>,
        graphics: String,
    },
}

#[derive(Serialize, Deserialize)]
enum GenerationProduct {
    RegionalCity(Box<city::PreparedCityProduct>),
    Scene(Box<scene::SceneProduct>),
    Building(Box<PreparedFacade>),
    Venue(Box<venue::PreparedVenue>),
    Grass(Box<landscape::GrassProduct>),
    Ground(Box<landscape::GroundProduct>),
}

impl GenerationJob {
    fn require_owner(&self, ticket: PreparationTicket) -> PreparationResult<()> {
        match self {
            Self::RegionalCity { .. } => ticket.require_owner(PresentationOwner::RegionalMap),
            Self::Building(_) => Ok(()),
            Self::Scene(_) | Self::Venue(_) | Self::Grass { .. } | Self::Ground { .. } => {
                ticket.require_owner(PresentationOwner::Scene)
            }
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod travel_fixtures;
