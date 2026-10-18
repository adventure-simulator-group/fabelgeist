//! Repeatable domestic heating specimens through production building recipes.
use super::*;

pub(super) fn fixture() -> Fixture {
    Fixture {
        buildings: BuildingFixture::HeatingReview,
        playable_spacing_metres: 25.0,
        ..super::fixture(
            "heating-review",
            "city",
            fabelgeist_determinism::Seed::from_u64(47_126),
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
    let mut buildings = [
        (
            BuildingArchetype::FachwerkCottage,
            fabelgeist_determinism::Seed::from_u64(42),
        ),
        (
            BuildingArchetype::FachwerkCottage,
            fabelgeist_determinism::Seed::from_u64(47),
        ),
        (
            BuildingArchetype::FachwerkCottage,
            fabelgeist_determinism::Seed::from_u64(101),
        ),
        (
            BuildingArchetype::HallHouse,
            fabelgeist_determinism::Seed::from_u64(42),
        ),
        (
            BuildingArchetype::HallHouse,
            fabelgeist_determinism::Seed::from_u64(47),
        ),
        (
            BuildingArchetype::HallHouse,
            fabelgeist_determinism::Seed::from_u64(101),
        ),
        (
            BuildingArchetype::HallHouse,
            fabelgeist_determinism::Seed::from_u64(u64::MAX),
        ),
        (
            BuildingArchetype::TownHouse,
            fabelgeist_determinism::Seed::from_u64(11),
        ),
        (
            BuildingArchetype::FachwerkMerchantHouse,
            fabelgeist_determinism::Seed::from_u64(0),
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (archetype, seed))| {
        let mut program = BuildingProgram::fixture(archetype, seed);
        if matches!(
            archetype,
            BuildingArchetype::TownHouse | BuildingArchetype::FachwerkMerchantHouse
        ) {
            program.domestic_heating = Some(
                adventuresim_building_generator::DomesticHeatingProgramme::HearthAndRearFedStove,
            );
        }
        Ok(TacticalBuildingPlacement {
            base_elevation_metres:
                adventuresim_tactical_core::city_layout::grounding::SupportElevation::ZERO,
            id: adventuresim_tactical_core::scene_input::SceneBuildingId(index as u64 + 1),
            program,
            centre_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                Vec2::ZERO,
            )?,
            orientation: BuildingOrientation::IDENTITY,
        })
    })
    .collect::<Result<Vec<_>, adventuresim_building_generator::spatial_geometry::GeometryError>>(
    )?;
    super::support::arrange_catalogue_grid(&mut buildings, 3)?;
    Ok(buildings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_heating_specimens_have_bare_playable_ground_beneath_them() {
        let input = super::super::build_fixture(fixture()).unwrap();
        assert_eq!(input.buildings.len(), 9);
        let programmes = input
            .buildings
            .iter()
            .map(|b| (b.id, b.program.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            programmes[7].0,
            adventuresim_tactical_core::scene_input::SceneBuildingId(8)
        );
        assert_eq!(programmes[7].1.archetype, BuildingArchetype::TownHouse);
        assert_eq!(
            programmes[7].1.seed,
            fabelgeist_determinism::Seed::from_u64(11)
        );
        assert_eq!(
            programmes[8].0,
            adventuresim_tactical_core::scene_input::SceneBuildingId(9)
        );
        assert_eq!(
            programmes[8].1.archetype,
            BuildingArchetype::FachwerkMerchantHouse
        );
        assert_eq!(
            programmes[8].1.seed,
            fabelgeist_determinism::Seed::from_u64(0)
        );
        let generated = input.generate().unwrap();
        for building in &generated.buildings {
            assert_eq!(
                building.placement.program,
                programmes[(building.placement.id.0 - 1) as usize].1
            );
        }
        for building in generated.buildings.iter().filter(|b| {
            b.placement.id >= adventuresim_tactical_core::scene_input::SceneBuildingId(8)
        }) {
            let half = building
                .collision
                .bounds
                .plan_half_extents()
                .unwrap()
                .metres();
            for z in [-0.9, 0.0, 0.9] {
                for x in [-0.9, 0.0, 0.9] {
                    let point = building.placement.centre_metres.metres()
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
