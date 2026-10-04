use super::*;

#[test]
fn approach_crosses_a_shared_street_segment_join_without_requiring_a_single_owner() {
    let left = CityPlotBounds {
        centre_metres: Vec2::new(-5.0, 0.0),
        dimensions_metres: Vec2::new(10.0, 6.0),
        orientation: BuildingOrientation::IDENTITY,
    };
    let right = CityPlotBounds {
        centre_metres: Vec2::new(5.0, 0.0),
        ..left
    };
    let outlines = |r: &[CityPlotBounds]| {
        r.iter()
            .map(|b| b.corners().map(Vec2::as_dvec2).to_vec())
            .collect::<Vec<_>>()
    };
    let edge = Vec2::new(0.0, 2.9);
    let direction = -Vec2::Y;
    let offset = Vec2::X * 0.5;
    assert!(
        (available_run(
            edge,
            direction,
            offset,
            &outlines(&[left, right]),
            2.9,
            0.001
        ) - 2.9)
            .abs()
            < 0.001
    );
    assert_eq!(
        available_run(edge, direction, offset, &outlines(&[left]), 2.9, 0.001),
        0.0
    );
    let gap = CityPlotBounds {
        centre_metres: Vec2::new(5.1, 0.0),
        ..right
    };
    assert_eq!(
        available_run(edge, direction, offset, &outlines(&[left, gap]), 2.9, 0.001),
        0.0
    );
    assert_eq!(
        available_run(
            edge,
            direction,
            offset,
            &outlines(&[right, left]),
            2.9,
            0.001
        ),
        available_run(
            edge,
            direction,
            offset,
            &outlines(&[left, right]),
            2.9,
            0.001
        )
    );
}

#[test]
fn repeated_clipping_endpoint_does_not_turn_zero_area_into_a_blocked_route() {
    let point = bevy::math::DVec2::new(-197.761, 198.91765);
    assert_eq!(minimum_width(&[point, point, point]), 0.0);
}

#[test]
fn population_6500_seed_42_doorway_join_retains_its_full_near_street_approach() {
    let value: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/doorway-junction-support.json"
    )))
    .unwrap();
    let property: crate::city_layout::CitySingleProperty =
        serde_json::from_value(value["property"].clone()).unwrap();
    let streets: Vec<CityStreetPatch> = serde_json::from_value(value["streets"].clone()).unwrap();
    let edge: Vec2 = serde_json::from_value(value["entrances"][0]["threshold"].clone()).unwrap();
    let direction: Vec2 = serde_json::from_value(value["entrances"][0]["outward"].clone()).unwrap();
    let footprint: Vec<Vec2> = serde_json::from_value(value["footprint"].clone()).unwrap();
    let mut regions = access_regions(property.plot, &streets, edge);
    regions.push(footprint.into_iter().map(Vec2::as_dvec2).collect());
    let offset = Vec2::new(direction.y, -direction.x) * 0.5;
    let run = available_run(edge, direction, offset, &regions, 4.0, 0.001);
    assert!((run - 3.0).abs() < 0.001, "run {run}");
}
