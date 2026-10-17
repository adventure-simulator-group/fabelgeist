//! Repeatable main-gable glazing specimens through production building recipes.
use super::*;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::GableReview,
        playable_spacing_metres: 12.5,
        ..super::fixture(
            "gable-review",
            "city",
            fabelgeist_determinism::Seed::from_u64(47_125),
            flat,
            |_, _| sample(TacticalSurface::Open, 0, 0, 0, 0),
            clear(),
        )
    }
}

pub(super) fn buildings() -> Result<
    Vec<TacticalBuildingPlacement>,
    adventuresim_building_generator::spatial_geometry::GeometryError,
> {
    [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ]
    .into_iter()
    .enumerate()
    .flat_map(|(row, archetype)| {
        [42, 47, 101]
            .map(fabelgeist_determinism::Seed::from_u64)
            .into_iter()
            .enumerate()
            .map(move |(column, seed)| {
                Ok(TacticalBuildingPlacement {
                    base_elevation_metres:
                        adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO,
                    id: adventuresim_tactical_core::scene_input::SceneBuildingId(
                        (row * 3 + column + 1) as u64,
                    ),
                    program: BuildingProgram::fixture(archetype, seed),
                    centre_metres:
                        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                            Vec2::new((column as f32 - 1.0) * 40.0, row as f32 * 45.0 - 22.5),
                        )?,
                    orientation: BuildingOrientation::IDENTITY,
                })
            })
    })
    .collect()
}
