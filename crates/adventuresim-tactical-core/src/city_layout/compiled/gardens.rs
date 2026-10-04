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

pub(super) fn envelope(
    front: &TacticalBuildingPlacement,
    recipe: &recipes::Recipe,
) -> CityPlotBounds {
    CityPlotBounds {
        centre_metres: front.centre_metres
            + front
                .orientation
                .local_to_world((recipe.render_min + recipe.render_max) * 0.5),
        dimensions_metres: recipe.render_max - recipe.render_min,
        orientation: front.orientation,
    }
}

pub(super) fn compile(
    seed: u64,
    lot: CityBuildingLot,
    front: &TacticalBuildingPlacement,
    recipe: &recipes::Recipe,
    streets: &[CityStreetPatch],
) -> Option<CityGarden> {
    if lot.service.is_some()
        || lot.has_rear_range()
        || GARDEN_SELECTION_DOMAIN
            .rng(seed, &[lot.id])
            .index(GARDEN_SELECTION_DIVISOR)
            != 0
    {
        return None;
    }
    let half = lot.footprint_metres * 0.5;
    let world = |p| lot.centre_metres + lot.orientation.local_to_world(p);
    let bounds = |p, dimensions_metres| CityPlotBounds {
        centre_metres: world(p),
        dimensions_metres,
        orientation: lot.orientation,
    };
    let side = half.x + plots::SIDE_PASSAGE_METRES * 0.5;
    let street_start = (1..=STREET_SEARCH_STEPS)
        .map(|step| {
            world(Vec2::new(
                side,
                -half.y - step as f32 * STREET_SEARCH_STEP_METRES,
            ))
        })
        .find(|point| streets.iter().any(|street| street.contains(*point)))?;
    let junction = world(Vec2::new(side, half.y + 3.0));
    let route = |start_metres, end_metres| CityAccessSegment {
        start_metres,
        end_metres,
        half_width_metres: ACCESS_HALF_WIDTH_METRES,
    };
    let garden = CityGarden {
        owner: CityPropertyId(lot.id),
        front_building_id: front.id,
        plot: CityPlotBounds::from(plots::reservation(lot)),
        cultivated_bounds: bounds(
            Vec2::new(0.0, half.y + plots::REAR_COURT_METRES * 0.5),
            Vec2::new(lot.footprint_metres.x, plots::REAR_COURT_METRES),
        ),
        beds: [1.5, 4.5]
            .map(|depth| {
                bounds(
                    Vec2::new(half.x - 1.75, half.y + depth),
                    Vec2::new(2.5, 1.2),
                )
            })
            .to_vec(),
        access: vec![
            route(street_start, junction),
            route(junction, world(Vec2::new(0.0, half.y + 3.0))),
            route(
                world(Vec2::new(0.0, half.y + 1.6)),
                world(Vec2::new(0.0, half.y + 5.3)),
            ),
        ],
        plants: vec![GardenPlantPlacement {
            id: GardenPlantId(lot.id),
            specimen: GardenSpecimen::CommonHazel,
            centre_metres: world(Vec2::new(-half.x + 1.5, half.y + 4.0)),
            orientation: lot.orientation,
            scale: GardenPlantScale::new(0.75),
        }],
    };
    (garden.validate_geometry(streets).is_ok()
        && garden.plot.contains(front.centre_metres)
        && garden.clears_building(envelope(front, recipe)))
    .then_some(garden)
}

pub(crate) fn validate_scene_gardens(
    gardens: &[CityGarden],
    buildings: &[crate::scene_input::GeneratedBuilding],
    distant: &[DistantBuildingPlacement],
) -> Result<(), String> {
    if gardens.is_empty() {
        return Ok(());
    }
    let check = |id, centre, bounds: CityPlotBounds| -> Result<(), String> {
        if gardens
            .iter()
            .filter(|g| g.front_building_id == id)
            .any(|g| !g.plot.contains(centre) || g.cultivated_bounds.contains(centre))
        {
            return Err("garden owner geometry lies outside its property".into());
        }
        if gardens.iter().any(|g| !g.clears_building(bounds)) {
            Err("garden working ground or plant intersects building geometry".into())
        } else {
            Ok(())
        }
    };
    for building in buildings {
        check(
            building.placement.id,
            building.placement.centre_metres,
            envelope(
                &building.placement,
                &recipes::Recipe::from_generated(building),
            ),
        )?;
    }
    let mut palette = recipes::RecipePalette::default();
    for building in distant {
        let recipe = palette
            .get(
                building.archetype,
                building.usage,
                building.service_size,
                building.seed,
            )
            .map_err(|error| error.to_string())?;
        check(
            building.id,
            building.centre_metres,
            envelope(
                &TacticalBuildingPlacement {
                    id: building.id,
                    program: recipe.program.clone(),
                    centre_metres: building.centre_metres,
                    orientation: building.orientation,
                },
                &recipe,
            ),
        )?;
    }
    Ok(())
}
