//! Regular pleat construction shared by textile relief and garment components.
use crate::{ClothPleatProfile, Millimeters, TextileRepeatCount};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothPleats {
    pub count: TextileRepeatCount,
    pub depth: Millimeters,
    pub profile: ClothPleatProfile,
}
