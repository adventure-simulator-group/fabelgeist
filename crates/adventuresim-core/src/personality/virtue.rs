use serde::{Deserialize, Serialize};

/// Chivalric deed vocabulary shared by authored developments and personality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "spacetimedb", runtime_catalog),
    derive(spacetimedb::SpacetimeType)
)]
#[serde(rename_all = "snake_case")]
pub enum ChivalricVirtue {
    Courage,
    Mercy,
    Faith,
    Justice,
    Courtesy,
    Loyalty,
    Prudence,
    Honesty,
}
