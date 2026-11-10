//! Canonical seven simple attributes, also used by authored checks.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "spacetimedb", runtime_catalog),
    derive(spacetimedb::SpacetimeType)
)]
#[serde(rename_all = "snake_case")]
pub enum SimpleAttribute {
    /// Heart strength, lung capacity, endurance for traveling.
    Endurance,
    /// Liver, spleen, immune system, toxin filtering.
    Immunity,
    /// Digestive system, food tolerance.
    Gut,
    /// Deep thinking; learning speed and mastery cap for intellectual skills.
    Intelligence,
    /// Quick decisions; learning speed and mastery cap for instinctive skills.
    Instinct,
    /// Visual acuity.
    Eyesight,
    /// Auditory perception.
    Hearing,
}
