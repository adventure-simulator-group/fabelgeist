//! Stable failure classifications for ecosystem and environment operations.
use crate::identity::{ModuleId, PlantId, SpeciesId, VineSpeciesId};
use crate::units::Years;
pub type Result<T> = std::result::Result<T, BotanyError>;

#[derive(Debug, thiserror::Error)]
pub enum BotanyError {
    #[error("regulation duration must be finite and positive, got {duration:?}")]
    InvalidStep { duration: Years },
    #[error("unknown botanical Table 4 row {letter}")]
    UnknownTableRow { letter: char },
    #[error("could not start the botanical simulation thread")]
    ThreadSpawn(#[source] std::io::Error),
    #[error("unknown woody species {species:?}")]
    UnknownSpecies { species: SpeciesId },
    #[error("unknown climbing species {species:?}")]
    UnknownVineSpecies { species: VineSpeciesId },
    #[error("the requested ground is blocked")]
    BlockedGround,
    #[error("environment returned {actual} orientations for {expected} requests")]
    OrientationCount { expected: usize, actual: usize },
    #[error("environment has {parents} parent entries for {modules} modules")]
    ParentCount { modules: usize, parents: usize },
    #[error("plant {plant:?} module {module:?} has no laid-out attachment")]
    MissingAttachment { plant: PlantId, module: ModuleId },
    #[error("botanical GPU operation failed")]
    Backend(#[from] anyhow::Error),
}
