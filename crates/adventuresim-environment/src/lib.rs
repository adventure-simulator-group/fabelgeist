//! Fabelgeist's living environments.
//!
//! Plants are grown, not placed: `fabelgeist-botany` simulates whole
//! ecosystems after Makowski et al.'s *Synthetic Silviculture* -- species
//! competing for light, seeding, adapting to climate and climbing the
//! buildings among them. This crate chooses what grows where. Its
//! `environment-viewer` binary (the `viewer` feature) watches a scene grow.

mod scene;
mod solo;

pub use scene::{SceneLookupError, scene_named};
pub use solo::SoloSpecies;
