//! Repeatable domestic heating specimens through production building recipes.
use super::*;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::HeatingReview,
        playable_spacing_metres: 12.5,
        ..super::fixture(
            "heating-review",
            "city",
            47_126,
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
    .chain(std::iter::once(TacticalBuildingPlacement {
        id: 7,
        program: BuildingProgram::fixture(BuildingArchetype::HallHouse, u64::MAX),
        centre_metres: Vec2::ZERO,
        orientation: BuildingOrientation::IDENTITY,
    }))
    .collect()
}
