//! Bounded exterior prototypes for buildings outside the tactical boundary.
use super::DistantBuildingPlacement;
use adventuresim_building_generator::{BuildingArchetype, BuildingProgram};
use adventuresim_world_schema::ProsperityTier;
use fabelgeist_determinism::StreamId;

const EXTERIOR_VARIATION: StreamId = StreamId::new("city.distant-exterior");

/// Geometry and finish select the same variant, rather than multiplying pools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistantBuildingVariant {
    Plain,
    Weathered,
    Decorated,
}

impl DistantBuildingVariant {
    const ALL: [Self; 3] = [Self::Plain, Self::Weathered, Self::Decorated];

    fn seed(self) -> u64 {
        match self {
            Self::Plain => 42,
            Self::Weathered => 47,
            Self::Decorated => 101,
        }
    }
}

impl DistantBuildingPlacement {
    pub fn exterior_variant(self) -> DistantBuildingVariant {
        let index = EXTERIOR_VARIATION
            .rng(self.seed, &[self.id])
            .index(DistantBuildingVariant::ALL.len());
        DistantBuildingVariant::ALL[index]
    }

    /// Occupation, lot identity and service capacity do not create new meshes.
    /// Poor settlements omit the ornate merchant-house family from the skyline.
    pub fn exterior_program(self) -> BuildingProgram {
        let archetype = match (self.archetype, self.prosperity) {
            (BuildingArchetype::ParishChurch, _)
                if self.occupied_program().church_program.is_some() =>
            {
                BuildingArchetype::Cathedral
            }
            (
                BuildingArchetype::FachwerkMerchantHouse,
                ProsperityTier::Subsistence | ProsperityTier::Modest,
            ) => BuildingArchetype::TownHouse,
            (archetype, _) => archetype,
        };
        BuildingProgram::fixture(archetype, self.exterior_variant().seed())
    }

    /// Uniformly shrink a shared exterior to fit its reserved plot. Never stretch
    /// normals, enlarge its footprint, or move the authored city placement.
    pub fn exterior_scale(self, exterior: &BuildingProgram) -> f32 {
        (self.occupied_program().plot_dimensions_metres() / exterior.plot_dimensions_metres())
            .min_element()
            .min(1.0)
    }
}

#[cfg(test)]
mod tests;
