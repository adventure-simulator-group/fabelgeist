//! Native duel setup and simulation failures retain their owning contracts.

use std::time::Duration;

use adventuresim_tactical_core::{animation::AnimationValueError, scene::TerrainAdmissionError};

pub type TacticalIterationResult<T> = Result<T, TacticalIterationError>;

#[derive(Debug, thiserror::Error)]
pub enum TacticalIterationError {
    #[error(transparent)]
    Animation(#[from] AnimationValueError),
    #[error(transparent)]
    Terrain(#[from] TerrainAdmissionError),
    #[error("navigation is unavailable during native duel setup")]
    NavigationUnavailable,
    #[error("navigation did not become ready within {limit:?} during native duel setup")]
    NavigationStartupTimedOut { limit: Duration },
}
