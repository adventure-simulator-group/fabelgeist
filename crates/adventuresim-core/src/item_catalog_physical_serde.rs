//! The authored catalogue keeps its original local X/Y/Z array Serde boundary.

use fabelgeist_geometry::CuboidDimensions;
use serde::{Deserialize, Serialize};

pub(super) fn serialize<S: serde::Serializer>(
    dimensions: &CuboidDimensions,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    dimensions.axis_metres().serialize(serializer)
}

pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<CuboidDimensions, D::Error> {
    let axes = <[f32; 3]>::deserialize(deserializer)?;
    CuboidDimensions::from_metres(axes).map_err(serde::de::Error::custom)
}
