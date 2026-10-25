//! The same civilian recipes used for matched exterior construction captures.
use super::*;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::FacadeReview,
        playable_spacing_metres: 12.5,
        ..super::fixture(
            "facade-review",
            "city",
            fabelgeist_determinism::Seed::from_u64(47_127),
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
        BuildingArchetype::FachwerkCottage,
        BuildingArchetype::HallHouse,
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, archetype)| {
        Ok(TacticalBuildingPlacement {
            base_elevation_metres:
                adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO,
            id: adventuresim_tactical_core::scene_input::SceneBuildingId(index as u64 + 1),
            program: BuildingProgram::fixture(
                archetype,
                fabelgeist_determinism::Seed::from_u64(42),
            ),
            centre_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                Vec2::new(
                    (index % 2) as f32 * 45.0 - 22.5,
                    (index / 2) as f32 * 45.0 - 22.5,
                ),
            )?,
            orientation: BuildingOrientation::IDENTITY,
        })
    })
    .chain([Ok(TacticalBuildingPlacement {
        base_elevation_metres:
            adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO,
        id: adventuresim_tactical_core::scene_input::SceneBuildingId(5),
        program: BuildingProgram::settlement(
            BuildingArchetype::HallHouse,
            Some(adventuresim_world_schema::settlement_buildings::BuildingUse::Dwelling),
            fabelgeist_determinism::Seed::from_u64(2),
        ),
        centre_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            Vec2::new(-22.5, 67.5),
        )?,
        orientation: BuildingOrientation::IDENTITY,
    })])
    .collect()
}
