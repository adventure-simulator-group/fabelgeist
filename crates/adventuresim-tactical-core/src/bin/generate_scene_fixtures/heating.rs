//! Repeatable domestic heating specimens through production building recipes.
use super::*;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::HeatingReview,
        playable_spacing_metres: 25.0,
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
    .chain(
        [
            (8, BuildingArchetype::TownHouse, 11, -25.0),
            (9, BuildingArchetype::FachwerkMerchantHouse, 0, 25.0),
        ]
        .into_iter()
        .map(|(id, archetype, seed, x)| {
            let mut program = BuildingProgram::fixture(archetype, seed);
            program.domestic_heating = Some(
                adventuresim_building_generator::DomesticHeatingProgramme::HearthAndRearFedStove,
            );
            TacticalBuildingPlacement {
                id,
                program,
                centre_metres: Vec2::new(x, 67.5),
                orientation: BuildingOrientation::IDENTITY,
            }
        }),
    )
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_heating_specimens_have_bare_playable_ground_beneath_them() {
        let input = super::super::build_fixture(fixture());
        let generated = input.generate().unwrap();
        for building in generated.buildings.iter().filter(|b| b.placement.id >= 8) {
            let half = building.collision.bounds.plan_half_extents();
            for z in [-0.9, 0.0, 0.9] {
                for x in [-0.9, 0.0, 0.9] {
                    let point = building.placement.centre_metres
                        + building
                            .placement
                            .orientation
                            .local_to_world(half * Vec2::new(x, z));
                    assert!(generated.terrain.height_at(point).is_some());
                    let ground = generated.ground.ground_at(point).unwrap();
                    assert_eq!(ground.cover, GroundCover::Bare);
                    assert_eq!(ground.cover_density_bps, 0);
                }
            }
        }
    }
}
