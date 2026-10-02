//! The same civilian recipes used for matched exterior construction captures.
use super::*;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::FacadeReview,
        playable_spacing_metres: 12.5,
        ..super::fixture(
            "facade-review",
            "city",
            47_127,
            flat,
            |_, _| sample(TacticalSurface::Open, 0, 0, 0, 0),
            clear(),
        )
    }
}

pub(super) fn buildings() -> Vec<TacticalBuildingPlacement> {
    [
        BuildingArchetype::FachwerkCottage,
        BuildingArchetype::HallHouse,
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, archetype)| TacticalBuildingPlacement {
        id: index as u64 + 1,
        program: BuildingProgram::fixture(archetype, 42),
        centre_metres: Vec2::new(
            (index % 2) as f32 * 45.0 - 22.5,
            (index / 2) as f32 * 45.0 - 22.5,
        ),
        orientation: BuildingOrientation::IDENTITY,
    })
    .chain([TacticalBuildingPlacement {
        id: 5,
        program: BuildingProgram::settlement(
            BuildingArchetype::HallHouse,
            Some(adventuresim_world_schema::settlement_buildings::BuildingUse::Dwelling),
            2,
        ),
        centre_metres: Vec2::new(-22.5, 67.5),
        orientation: BuildingOrientation::IDENTITY,
    }])
    .collect()
}
