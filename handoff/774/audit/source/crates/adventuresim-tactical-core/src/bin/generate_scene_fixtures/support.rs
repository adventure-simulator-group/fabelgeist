//! Author explicit reservations for catalogue buildings before support planning.
use adventuresim_tactical_core::scene_input::GeneratedBuildingRecipes;
use adventuresim_tactical_core::{city_layout::*, prelude::*};
use bevy::math::Vec2;
use std::collections::BTreeSet;

/// Catalogue layouts author their own sites. Generated cities already retain
/// authoritative reservations and never pass through this authoring operation.
pub(super) fn declare_catalogue_properties(
    layout: &mut CitySceneLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    let bound: BTreeSet<_> = layout
        .compounds
        .iter()
        .flat_map(|p| [p.front_building_id, p.rear_building_id])
        .chain(layout.single_properties.iter().map(|p| p.building_id))
        .collect();
    let mut recipes = GeneratedBuildingRecipes::default();
    for placement in layout
        .playable
        .iter()
        .cloned()
        .chain(
            layout
                .distant
                .iter()
                .copied()
                .map(TacticalBuildingPlacement::from),
        )
        .filter(|p| !bound.contains(&p.id))
    {
        let recipe = recipes
            .get_or_generate(&placement.program)
            .unwrap_or_else(|error| {
                panic!(
                    "authored catalogue building {}, programme {:?}: {error}",
                    placement.id, placement.program
                )
            });
        let plot = layout
            .gardens
            .iter()
            .find(|g| g.front_building_id == placement.id)
            .map(|garden| garden.plot)
            .map(Ok)
            .unwrap_or_else(|| {
                // Art catalogues reserve a doorway approach on each side.
                // The support planner still clips each approach and proves
                // complete bearings, access, source coverage and ownership.
                let approach = CompoundGradingPolicy::bounded_settlement()
                    .street_apron
                    .dimensions_metres()
                    .y;
                CityPlotBounds::new(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                        placement.centre_metres.metres(),
                    )?,
                    adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                        recipe
                            .collision
                            .bounds
                            .plan_half_extents()
                            .expect("admitted review bounds")
                            .metres()
                            * 2.0
                            + Vec2::splat(approach * 2.0),
                    )?,
                    placement.orientation,
                )
                .map_err(|error| -> Box<dyn std::error::Error> { error.into() })
            })?;
        layout.single_properties.push(CitySingleProperty {
            id: CityPropertyId(placement.id.0),
            building_id: placement.id,
            plot,
        });
    }
    Ok(())
}

/// Review grids space whole compiled property reservations, including access,
/// instead of assuming that authored building centres establish separation.
pub(super) fn arrange_catalogue_grid(
    buildings: &mut [TacticalBuildingPlacement],
    columns: usize,
) -> Result<(), adventuresim_building_generator::spatial_geometry::GeometryError> {
    const REVIEW_SITE_GAP_METRES: f32 = 4.0;
    assert!(columns > 0, "catalogue grid has at least one column");
    let approach = CompoundGradingPolicy::bounded_settlement()
        .street_apron
        .dimensions_metres()
        .y;
    let mut recipes = GeneratedBuildingRecipes::default();
    let mut spacing = Vec2::ZERO;
    for building in buildings.iter() {
        let recipe = recipes
            .get_or_generate(&building.program)
            .expect("accepted review building programme");
        let half = recipe
            .collision
            .bounds
            .plan_half_extents()
            .expect("admitted review bounds")
            .metres()
            + Vec2::splat(approach);
        let extent = [Vec2::new(half.x, half.y), Vec2::new(half.x, -half.y)]
            .map(|p| building.orientation.local_to_world(p).abs())
            .into_iter()
            .fold(Vec2::ZERO, Vec2::max)
            * 2.0;
        spacing = spacing.max(extent);
    }
    spacing += Vec2::splat(REVIEW_SITE_GAP_METRES);
    let rows = buildings.len().div_ceil(columns);
    for (index, building) in buildings.iter_mut().enumerate() {
        building.centre_metres =
            adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                Vec2::new(
                    (index % columns) as f32 - (columns - 1) as f32 * 0.5,
                    (index / columns) as f32 - rows.saturating_sub(1) as f32 * 0.5,
                ) * spacing,
            )?;
    }
    Ok(())
}

/// Generate support from an explicitly authored, current-format draft.
/// This is authoring only: production loading rejects unbound occupied scenes.
pub(super) fn ground_source_fixture(
    path: &std::path::Path,
) -> Result<TacticalSceneInput, Box<dyn std::error::Error>> {
    let input: TacticalSceneInput = serde_json::from_slice(&std::fs::read(path)?)?;
    assert!(
        matches!(input.source, SceneSource::SyntheticFixture(_)),
        "catalogue authoring cannot synthesize imported property reservations"
    );
    let mut layout = CitySceneLayout {
        playable: input.buildings.clone(),
        distant: input.distant_buildings.clone(),
        compounds: input.compounds.clone(),
        gardens: input.gardens.clone(),
        streets: input.streets.clone(),
        yards: input.yards.clone(),
        parishes: input.parishes.clone(),
        ..Default::default()
    };
    declare_catalogue_properties(&mut layout)?;
    let original = input.clone();
    let accepted =
        input.ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())?;
    for (before, after) in original.buildings.iter().zip(&accepted.buildings) {
        assert_eq!(before.id, after.id);
        assert_eq!(before.program, after.program);
        assert_eq!(before.centre_metres, after.centre_metres);
        assert_eq!(before.orientation, after.orientation);
    }
    for (before, after) in original
        .distant_buildings
        .iter()
        .zip(&accepted.distant_buildings)
    {
        let mut before = *before;
        before.base_elevation_metres = after.base_elevation_metres;
        assert_eq!(before, *after);
    }
    assert_eq!(original.properties, accepted.properties);
    assert_eq!(original.establishments, accepted.establishments);
    Ok(accepted)
}
