use adventuresim_core::settlement_property::HomeCapacity;
use std::num::NonZeroU32;

/// A physical urban dwelling class with the exact generated footprint used to pack frontages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CityHouseClass {
    Cottage,
    CraftTownHouse,
    HallHouse,
    MerchantHouse,
}

impl CityHouseClass {
    pub const fn housing_tier(self) -> adventuresim_core::courtship::HousingTier {
        use adventuresim_core::courtship::HousingTier;
        match self {
            Self::Cottage => HousingTier::Cheap,
            Self::CraftTownHouse | Self::HallHouse => HousingTier::Moderate,
            Self::MerchantHouse => HousingTier::Fancy,
        }
    }

    pub const fn archetype(self) -> adventuresim_building_generator::BuildingArchetype {
        use adventuresim_building_generator::BuildingArchetype;
        match self {
            Self::Cottage => BuildingArchetype::FachwerkCottage,
            Self::CraftTownHouse => BuildingArchetype::TownHouse,
            Self::HallHouse => BuildingArchetype::HallHouse,
            Self::MerchantHouse => BuildingArchetype::FachwerkMerchantHouse,
        }
    }

    pub fn from_archetype(
        archetype: adventuresim_building_generator::BuildingArchetype,
    ) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|class| class.archetype() == archetype)
    }

    pub const ALL: [Self; 4] = [
        Self::Cottage,
        Self::CraftTownHouse,
        Self::HallHouse,
        Self::MerchantHouse,
    ];

    /// Authored east/north dimensions enter the common positive geometry owner.
    pub fn footprint(
        self,
    ) -> adventuresim_building_generator::spatial_geometry::GeometryResult<
        adventuresim_building_generator::spatial_geometry::PlanDimensions,
    > {
        use adventuresim_building_generator::spatial_geometry::PlanDimensions;
        use bevy::math::Vec2;
        PlanDimensions::from_metres(match self {
            Self::Cottage => Vec2::new(10.5, 12.0),
            Self::CraftTownHouse => Vec2::new(9.0, 15.0),
            Self::HallHouse => Vec2::new(13.5, 19.5),
            Self::MerchantHouse => Vec2::new(12.0, 16.5),
        })
    }

    pub const fn resident_capacity(self) -> HomeCapacity {
        match self {
            Self::Cottage => HomeCapacity::new(NonZeroU32::MIN.saturating_add(5)),
            Self::CraftTownHouse => HomeCapacity::new(NonZeroU32::MIN.saturating_add(12)),
            Self::HallHouse => HomeCapacity::new(NonZeroU32::MIN.saturating_add(15)),
            Self::MerchantHouse => HomeCapacity::new(NonZeroU32::MIN.saturating_add(29)),
        }
    }
}
