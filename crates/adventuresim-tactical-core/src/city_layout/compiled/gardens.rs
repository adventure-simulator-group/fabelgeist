//! Accept owned gardens only after the household's physical recipe is resolved.
use super::*;
use crate::city_layout::gardens::{
    GardenPlantId, GardenPlantPlacement, GardenPlantScale, GardenSpecimen,
};
const GARDEN_SELECTION_DOMAIN: StreamId = StreamId::new("city.garden-presence");
const GARDEN_SELECTION_DIVISOR: usize = 3;
const ACCESS_HALF_WIDTH_METRES: f32 = 0.4;
const STREET_SEARCH_STEP_METRES: f32 = 0.25;
const STREET_SEARCH_STEPS: usize = 24;

#[derive(Debug, thiserror::Error)]
pub enum GardenClearanceError {
    #[error("garden owner building {building} geometry lies outside its property")]
    OwnerGeometry {
        property: CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
    },
    #[error("garden working ground or plant intersects building {building}")]
    BuildingIntersection {
        property: CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
    },
    #[error("garden clearance building {building}: {source}")]
    Recipe {
        building: crate::scene_input::SceneBuildingId,
        #[source]
        source: super::CityCompileError,
    },
}

pub(super) fn envelope(
    front: &TacticalBuildingPlacement,
    recipe: &recipes::Recipe,
) -> CityCompileResult<CityPlotBounds> {
    Ok(CityPlotBounds::new(
        crate::scene_coordinates::ScenePlanPoint::try_from(
            front.centre_metres.metres()
                + front.orientation.local_to_world(
                    (recipe.render_min_metres() + recipe.render_max_metres()) * 0.5,
                ),
        )?,
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
            recipe.render_max_metres() - recipe.render_min_metres(),
        )?,
        front.orientation,
    )?)
}

pub(super) fn compile(
    seed: fabelgeist_determinism::Seed,
    lot: CityBuildingLot,
    front: &TacticalBuildingPlacement,
    recipe: &recipes::Recipe,
    streets: &[CityStreetPatch],
) -> CityCompileResult<Option<CityGarden>> {
    if lot.service.is_some()
        || lot.has_rear_range()
        || GARDEN_SELECTION_DOMAIN
            .rng(seed, &[lot.id.0])
            .index(GARDEN_SELECTION_DIVISOR)
            != 0
    {
        return Ok(None);
    }
    let Some(garden) = authored_garden(lot, front.id, streets)? else {
        return Ok(None);
    };
    if let Err(issue) = garden.validate_geometry(streets) {
        if matches!(issue, crate::city_layout::gardens::GardenIssue::Specimen(_)) {
            return Err(CityCompileError::Packing {
                property: lot.id,
                issue: CityPackingIssue::Garden { issue },
            });
        }
        return Ok(None);
    }
    Ok((garden.plot.contains(front.centre_metres.metres())
        && garden.clears_building(envelope(front, recipe)?))
    .then_some(garden))
}

pub(crate) fn validate_scene_gardens(
    gardens: &[CityGarden],
    buildings: &[crate::scene_input::GeneratedBuilding],
    distant: &[DistantBuildingPlacement],
) -> Result<(), GardenClearanceError> {
    if gardens.is_empty() {
        return Ok(());
    }
    let check = |id, centre, bounds: CityPlotBounds| -> Result<(), GardenClearanceError> {
        if let Some(garden) = gardens
            .iter()
            .filter(|g| g.front_building_id == id)
            .find(|g| !g.plot.contains(centre) || g.cultivated_bounds.contains(centre))
        {
            return Err(GardenClearanceError::OwnerGeometry {
                property: garden.owner,
                building: id,
            });
        }
        if let Some(garden) = gardens.iter().find(|g| !g.clears_building(bounds)) {
            return Err(GardenClearanceError::BuildingIntersection {
                property: garden.owner,
                building: id,
            });
        }
        Ok(())
    };
    for building in buildings {
        check(
            building.placement.id,
            building.placement.centre_metres.metres(),
            envelope(
                &building.placement,
                &recipes::Recipe::from_generated(building).map_err(|source| {
                    GardenClearanceError::Recipe {
                        building: building.placement.id,
                        source: source.into(),
                    }
                })?,
            )
            .map_err(|source| GardenClearanceError::Recipe {
                building: building.placement.id,
                source,
            })?,
        )?;
    }
    let mut palette = recipes::CityRecipePalette::default();
    for building in distant {
        let recipe = palette
            .get(
                building.archetype,
                building.usage,
                building.service_size,
                building.seed,
            )
            .map_err(|source| GardenClearanceError::Recipe {
                building: building.id,
                source,
            })?;
        check(
            building.id,
            building.centre_metres.metres(),
            envelope(
                &TacticalBuildingPlacement {
                    base_elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
                    id: building.id,
                    program: recipe.program.clone(),
                    centre_metres: building.centre_metres,
                    orientation: building.orientation,
                },
                &recipe,
            )
            .map_err(|source| GardenClearanceError::Recipe {
                building: building.id,
                source,
            })?,
        )?;
    }
    Ok(())
}

/// Construct the complete authored reservation before accepting its working space.
fn authored_garden(
    lot: CityBuildingLot,
    front_building_id: crate::scene_input::SceneBuildingId,
    streets: &[CityStreetPatch],
) -> CityCompileResult<Option<CityGarden>> {
    let half = lot.footprint_metres.metres() * 0.5;
    let world = |p| lot.centre_metres.metres() + lot.orientation.local_to_world(p);
    let bounds = |p,
                  dimensions_metres|
     -> adventuresim_building_generator::spatial_geometry::GeometryResult<
        CityPlotBounds,
    > {
        CityPlotBounds::new(
            crate::scene_coordinates::ScenePlanPoint::try_from(world(p))?,
            adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                dimensions_metres,
            )?,
            lot.orientation,
        )
    };
    let side = half.x + plots::SIDE_PASSAGE_METRES * 0.5;
    let Some(street_start) = (1..=STREET_SEARCH_STEPS)
        .map(|step| {
            world(Vec2::new(
                side,
                -half.y - step as f32 * STREET_SEARCH_STEP_METRES,
            ))
        })
        .find(|point| streets.iter().any(|street| street.contains(*point)))
    else {
        return Ok(None);
    };
    let junction = world(Vec2::new(side, half.y + 3.0));
    let route = |start_metres,
                 end_metres|
     -> adventuresim_building_generator::spatial_geometry::GeometryResult<
        CityAccessSegment,
    > {
        CityAccessSegment::new(
            crate::scene_coordinates::ScenePlanPoint::try_from(start_metres)?,
            crate::scene_coordinates::ScenePlanPoint::try_from(end_metres)?,
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                ACCESS_HALF_WIDTH_METRES,
            )?,
        )
    };
    let garden = CityGarden {
        owner: lot.id,
        front_building_id,
        plot: plots::reservation(lot)?,
        cultivated_bounds: bounds(
            Vec2::new(0.0, half.y + plots::REAR_COURT_METRES * 0.5),
            Vec2::new(lot.footprint_metres.metres().x, plots::REAR_COURT_METRES),
        )?,
        beds: [1.5, 4.5]
            .map(|depth| {
                bounds(
                    Vec2::new(half.x - 1.75, half.y + depth),
                    Vec2::new(2.5, 1.2),
                )
            })
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
        access: vec![
            route(street_start, junction)?,
            route(junction, world(Vec2::new(0.0, half.y + 3.0)))?,
            route(
                world(Vec2::new(0.0, half.y + 1.6)),
                world(Vec2::new(0.0, half.y + 5.3)),
            )?,
        ],
        plants: vec![GardenPlantPlacement {
            id: GardenPlantId(lot.id.0),
            specimen: GardenSpecimen::CommonHazel,
            centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(world(Vec2::new(
                -half.x + 1.5,
                half.y + 4.0,
            )))?,
            orientation: lot.orientation,
            scale: GardenPlantScale::STANDARD,
        }],
    };
    Ok(Some(garden))
}
