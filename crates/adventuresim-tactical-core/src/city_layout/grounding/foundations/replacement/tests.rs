use super::*;
use crate::city_layout::grounding::tests::Fixture;

#[test]
fn goslar_965_owned_apron_join_leaves_no_source_sliver_across_the_walkable_route() {
    let fixture = Fixture::load_965();
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let terrain = BoundedPropertyTerrain::compile(
        &plan,
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    let entry = plan.street_entry.as_ref().unwrap();
    let join = entry.reservation.centre_metres()
        + entry.reservation.orientation().local_to_world(Vec2::Y)
            * entry.reservation.dimensions_metres().y
            * 0.5;
    let guard = CityPlotBounds::new(
        crate::scene_coordinates::ScenePlanPoint::try_from(join).unwrap(),
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(Vec2::new(
            0.8, 0.02,
        ))
        .unwrap(),
        entry.reservation.orientation(),
    )
    .unwrap()
    .corners()
    .map(|p| Vec3::new(p.x, 0.0, p.y));
    let guards = [
        [guard[0], guard[1], guard[2]],
        [guard[0], guard[2], guard[3]],
    ]
    .map(|points| GroundTriangle::new(points).unwrap());
    let area: f64 = terrain
        .natural_triangles
        .iter()
        .map(|points| {
            let natural = GroundTriangle::new(*points).unwrap();
            guards
                .iter()
                .map(|guard| super::super::geometry::area(&natural.intersection(guard)))
                .sum::<f64>()
        })
        .sum();
    assert!(
        area < 1e-6,
        "source sliver area {area} m² remains in the owned join"
    );
    assert_eq!(entry.reservation.dimensions_metres(), Vec2::new(1.0, 4.0));
    assert_eq!(plan.reservation(), fixture.property.plot);
}
