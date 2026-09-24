//! Gameplay chassis identity attached to the canonical precise recipe.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ComponentRole {
    Structure,
    Grip,
    Guard,
    Socket,
    Head,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponDesign {
    pub catalog_id: String,
    pub recipe: crate::recipe::Recipe,
}
