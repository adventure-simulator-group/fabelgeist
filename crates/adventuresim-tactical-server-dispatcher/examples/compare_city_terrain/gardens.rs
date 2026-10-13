//! Complete working regions are compared before and after bounded grading.
use super::*;
use adventuresim_tactical_core::{
    city_layout::{CityGarden, CityPlotBounds, CompoundGradingPolicy},
    combat_config::TacticalCombatConfig,
};
use serde::Serialize;

// Native scene east/north metres for numerical area comparisons. The index is
// the source collection ordinal emitted in the diagnostic JSON, not ownership.
struct GardenRegionOutline {
    kind: GardenRegion,
    index: Option<usize>,
    outline: Vec<Vec2>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum GardenRegion {
    SoilEnvelope,
    Bed,
    Route,
    PlantHull,
}

pub(super) fn measure(
    input: &TacticalSceneInput,
    source: &GeographicSurface,
    terrain: &SceneTerrain,
) -> Result<Value, Box<dyn std::error::Error>> {
    let composed = terrain
        .physical_geographic_surface()
        .ok_or("production terrain has no admitted support surface")?;
    let gardens: Vec<_> = input.gardens.iter().map(|garden| {
        let regions: Vec<_> = outlines(garden)?.into_iter().map(|region|
            measure_region(&region, source, &composed)).collect::<Result<Vec<_>,_>>()?;
        let roots = adventuresim_tactical_core::scene_input::SceneGarden::project(garden.clone(), terrain);
        Ok(json!({"property_id":garden.owner,"member_building_ids":[garden.front_building_id],"regions":regions,
            "geometry_validation":garden.validate_geometry(&input.streets),
            "root_projection": match roots { Ok(roots) => json!({"pass":true,"plant_support":roots.plant_support()}),
                Err(error) => json!({"pass":false,"error":error.to_string()}) }}))
    }).collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    Ok(
        json!({"scope":"Complete soil envelope, bed, route and plant-hull triangle intersections, before and after grading. A soil envelope can include its front building's occupied footprint; working beds, lanes and planting hulls must be clear. Construction grade is recorded separately from the authored controller's walking limit. Complete unchanged-source route comparisons bound introduced ledges; controller traversal and collision clearance are separate acceptance checks.","observations":gardens}),
    )
}

fn outlines(garden: &CityGarden) -> Result<Vec<GardenRegionOutline>, Box<dyn std::error::Error>> {
    let routes = garden.access.iter().enumerate().map(|(index, route)| {
        let tangent = route.end_metres() - route.start_metres();
        let bounds = CityPlotBounds::new(adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from((route.start_metres() + route.end_metres()) * 0.5)?,adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(Vec2::new(
                tangent.length() + route.half_width_metres() * 2.0,
                route.half_width_metres() * 2.0,
            ))?,BuildingOrientation::from_frontage_tangent(tangent)
                .ok_or(adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection)?)?;
        Ok(GardenRegionOutline { kind: GardenRegion::Route, index: Some(index), outline: bounds.corners().to_vec() })
    }).collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let plants = garden
        .plants
        .iter()
        .enumerate()
        .map(|(i, plant)| {
            Ok(GardenRegionOutline {
                kind: GardenRegion::PlantHull,
                index: Some(i),
                outline: plant
                    .world_hull()?
                    .into_iter()
                    .map(adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::metres)
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    Ok(std::iter::once(GardenRegionOutline {
        kind: GardenRegion::SoilEnvelope,
        index: None,
        outline: garden.cultivated_bounds.corners().to_vec(),
    })
    .chain(
        garden
            .beds
            .iter()
            .enumerate()
            .map(|(i, bed)| GardenRegionOutline {
                kind: GardenRegion::Bed,
                index: Some(i),
                outline: bed.corners().to_vec(),
            }),
    )
    .chain(routes)
    .chain(plants)
    .collect())
}

fn measure_region(
    region: &GardenRegionOutline,
    source: &GeographicSurface,
    composed: &GeographicSurface,
) -> Result<Value, Box<dyn std::error::Error>> {
    let GardenRegionOutline {
        kind,
        index,
        outline,
    } = region;
    let contact = f64::from(
        CompoundGradingPolicy::bounded_settlement()
            .limits
            .contact_tolerance_metres(),
    );
    let perimeter: f64 = outline
        .iter()
        .zip(outline.iter().cycle().skip(1))
        .take(outline.len())
        .map(|(a, b)| a.as_dvec2().distance(b.as_dvec2()))
        .sum();
    let area_bound = perimeter * CityPlotBounds::COORDINATE_TOLERANCE_METRES;
    let admitted =
        adventuresim_tactical_core::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
            outline
                .iter()
                .copied()
                .map(adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        )?;
    let measurement = source.measure_region(&admitted);
    let composed_measurement = composed.measure_region(&admitted);
    let comparison = source.compare_in_outline(composed, &admitted);
    let motor = TacticalCombatConfig::default().movement.motor;
    let walk_grade = f64::from(motor.maximum_walkable_slope_degrees)
        .to_radians()
        .tan();
    Ok(
        json!({"kind":kind,"element_index":index,"outline_metres":outline,
        "source_measurement":measurement,"composed_measurement":composed_measurement,
        "complete_surface_comparison":comparison,
        "contact_bound_metres":contact,"coverage_bound_square_metres":area_bound,
        "construction_limits":CompoundGradingPolicy::bounded_settlement().limits,
        "maximum_walkable_grade":walk_grade,
        "composed_grade_within_controller_bound":composed_measurement.is_some_and(|m| m.maximum_grade.ratio() <= walk_grade),
        "composed_coverage_complete":composed_measurement.is_some_and(|m|
            (m.required_area_square_metres.square_metres()-m.covered_area_square_metres.square_metres()).abs() <= area_bound),
        "source_coverage_complete":measurement.is_some_and(|m|
            (m.required_area_square_metres.square_metres()-m.covered_area_square_metres.square_metres()).abs() <= area_bound),
        "unchanged_source_within_contact_bound":comparison.is_some_and(|m|
            m.minimum.difference_metres.metres().abs() <= contact && m.maximum.difference_metres.metres().abs() <= contact
                && (m.required_area_square_metres.square_metres()-m.covered_area_square_metres.square_metres()).abs() <= area_bound)}),
    )
}
