//! Captured property identity must resolve to the actual generated home.
use super::*;
use bevy::math::Vec2;

pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    let Some(catalog) = &input.properties else {
        if matches!(input.source, SceneSource::ImportedPackage(_)) && !input.streets.is_empty() {
            return invalid(SceneValidationError::MissingHomeCatalog);
        }
        return Ok(());
    };
    catalog
        .validate(&catalog.settlement_id, catalog.population)
        .map_err(|error| SceneInputError::Validation(SceneValidationError::HomeCatalog(error)))?;
    for home in &catalog.homes {
        let placement = input
            .buildings
            .iter()
            .find(|building| building.id == crate::scene_input::SceneBuildingId(home.building_id))
            .map(|building| {
                (
                    building.centre_metres,
                    building.orientation,
                    building.program.clone(),
                )
            })
            .or_else(|| {
                input
                    .distant_buildings
                    .iter()
                    .find(|building| {
                        building.id == crate::scene_input::SceneBuildingId(home.building_id)
                    })
                    .map(|building| {
                        (
                            building.centre_metres,
                            building.orientation,
                            building.occupied_program(),
                        )
                    })
            });
        let Some((centre, orientation, program)) = placement else {
            return invalid(SceneValidationError::MissingHome {
                home: home.id.clone(),
                building: crate::scene_input::SceneBuildingId(home.building_id),
            });
        };
        if centre.metres() != Vec2::new(home.east_metres, home.north_metres)
            || orientation.yaw_radians() != home.yaw_radians
            || program.usage
                != Some(adventuresim_world_schema::settlement_buildings::BuildingUse::Dwelling)
            || f32::from(program.footprint.dimensions().0)
                * adventuresim_building_generator::CELL_SIZE_METRES
                != home.width_metres
            || f32::from(program.footprint.dimensions().1)
                * adventuresim_building_generator::CELL_SIZE_METRES
                != home.depth_metres
        {
            return invalid(SceneValidationError::HomeBinding {
                home: home.id.clone(),
                building: crate::scene_input::SceneBuildingId(home.building_id),
                location: Some(centre),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::settlement_property::{GeneratedHome, HousingMarketReserve, PropertyId};

    #[test]
    fn home_binding_rejects_missing_moved_rotated_or_resized_buildings() {
        let mut input = super::super::tests::fixture();
        let settlement = "physical-home-test";
        let mut market = HousingMarketReserve::default();
        let classes = [
            crate::city_layout::CityHouseClass::Cottage,
            crate::city_layout::CityHouseClass::CraftTownHouse,
            crate::city_layout::CityHouseClass::MerchantHouse,
            crate::city_layout::CityHouseClass::Cottage,
        ];
        let homes = classes
            .into_iter()
            .enumerate()
            .map(|(index, class)| {
                let building = DistantBuildingPlacement {
                    id: (index as u64 + 1).into(),
                    prosperity: adventuresim_world_schema::ProsperityTier::Subsistence,
                    archetype: class.archetype(),
                    usage: Some(
                        adventuresim_world_schema::settlement_buildings::BuildingUse::Dwelling,
                    ),
                    service_size: None,
                    seed: 42.into(),
                    centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(
                        index as f32 * 30.0,
                        20.0,
                    ))
                    .unwrap(),
                    base_elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
                    orientation: BuildingOrientation::IDENTITY,
                };
                input.distant_buildings.push(building);
                GeneratedHome {
                    id: PropertyId::new(settlement, building.id.0).unwrap(),
                    building_id: building.id.0,
                    tier: class.housing_tier(),
                    resident_capacity: class.resident_capacity(),
                    market_reserve: market.reserve(class.housing_tier()),
                    east_metres: building.centre_metres.metres().x,
                    north_metres: building.centre_metres.metres().y,
                    yaw_radians: 0.0,
                    width_metres: class.frontage_width_metres(),
                    depth_metres: class.depth_metres(),
                }
            })
            .collect();
        input.properties = Some(
            adventuresim_core::settlement_property::GeneratedHomeCatalog {
                settlement_id: settlement.into(),
                seed: adventuresim_core::settlement_population::settlement_building_seed(
                    settlement,
                ),
                population: 1,
                homes,
            },
        );
        validate(&input).unwrap();
        let original = input.clone();
        input.distant_buildings.pop();
        assert!(validate(&input).is_err());
        input = original.clone();
        input.distant_buildings[0].centre_metres = input.distant_buildings[0]
            .centre_metres
            .translated(crate::scene_coordinates::PlanDisplacement::from_metres(Vec2::X).unwrap())
            .unwrap();
        assert!(validate(&input).is_err());
        input = original.clone();
        input.distant_buildings[0].orientation = BuildingOrientation::from_radians(0.5).unwrap();
        assert!(validate(&input).is_err());
        input = original;
        input.properties.as_mut().unwrap().homes[0].width_metres += 1.0;
        assert!(validate(&input).is_err());
    }
}
