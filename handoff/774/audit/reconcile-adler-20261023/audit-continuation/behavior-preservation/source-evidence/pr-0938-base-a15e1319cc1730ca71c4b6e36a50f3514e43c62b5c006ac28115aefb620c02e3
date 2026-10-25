use super::{EquipmentLocation, OccupancyRequirement};
use serde::{Deserialize, Serialize};

/// Independent mounting zones within a limb's broad equipment input location.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "spacetimedb", runtime_catalog),
    derive(spacetimedb::SpacetimeType)
)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentFitZone {
    UpperArm,
    Elbow,
    Forearm,
    Thigh,
    Knee,
    Shin,
}

impl EquipmentFitZone {
    pub const fn fits_location(self, location: EquipmentLocation) -> bool {
        matches!(
            (self, location),
            (
                Self::UpperArm | Self::Elbow | Self::Forearm,
                EquipmentLocation::LeftArm | EquipmentLocation::RightArm
            ) | (
                Self::Thigh | Self::Knee | Self::Shin,
                EquipmentLocation::LeftLeg | EquipmentLocation::RightLeg
            )
        )
    }
}

impl OccupancyRequirement {
    /// Channel layering is independent of anatomical fit. A whole-location
    /// reservation conflicts with every zone in that location.
    pub fn conflicts_with(self, other: Self) -> bool {
        self.location == other.location
            && self.channel == other.channel
            && (self.channel.singleton_per_location() || self.order == other.order)
            && (self.fit_zone.is_none()
                || other.fit_zone.is_none()
                || self.fit_zone == other.fit_zone)
    }
}
