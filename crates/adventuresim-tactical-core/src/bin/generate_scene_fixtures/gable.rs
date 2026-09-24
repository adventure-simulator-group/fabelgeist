//! Repeatable main-gable glazing specimens through production building recipes.
use super::*;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::GableReview,
        playable_spacing_metres: 12.5,
        ..super::fixture(
            "gable-review",
            "city",
            47_125,
            flat,
            |_, _| sample(TacticalSurface::Open, 0, 0, 0, 0),
            clear(),
        )
    }
}

pub(super) fn buildings() -> Vec<TacticalBuildingPlacement> {
    [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ]
    .into_iter()
    .enumerate()
    .flat_map(|(row, archetype)| {
        [42, 47, 101]
            .into_iter()
            .enumerate()
            .map(move |(column, seed)| TacticalBuildingPlacement {
                id: (row * 3 + column + 1) as u64,
                program: BuildingProgram::fixture(archetype, seed),
                centre_metres: Vec2::new((column as f32 - 1.0) * 40.0, row as f32 * 45.0 - 22.5),
                orientation: BuildingOrientation::IDENTITY,
            })
    })
    .collect()
}
