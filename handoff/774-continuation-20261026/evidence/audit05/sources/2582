//! Authored anatomical surface regions and bounded garment coverage.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentSurfaceSpan {
    /// Ordered proximal-to-distal anatomical chain. More than one region makes
    /// a continuous span across multiple bones.
    pub regions: Vec<EquipmentAnatomicalRegion>,
    /// Which end remains fixed while the other end is clipped to `coverage`.
    pub anchor: SurfaceAnchor,
    /// Fraction of the combined region-chain length retained.
    pub coverage: f32,
}

/// Maximum authored region segments in one placement's surface description.
/// Keeps transient combat coverage bounded while retaining disconnected patches.
pub const MAX_EQUIPMENT_SURFACE_SEGMENTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceAnchor {
    Proximal,
    Distal,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentAnatomicalRegion {
    Head,
    Neck,
    Chest,
    LeftAxilla,
    RightAxilla,
    Stomach,
    Groin,
    LeftUpperArm,
    LeftForearm,
    RightUpperArm,
    RightForearm,
    LeftThigh,
    LeftLowerLeg,
    RightThigh,
    RightLowerLeg,
}
