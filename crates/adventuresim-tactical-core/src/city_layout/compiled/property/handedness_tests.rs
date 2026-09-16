use super::*;

#[test]
fn both_outer_passages_preserve_full_court_routes_and_inward_gate_sweeps() {
    let city = CitySite::central_german_market_town().generate(
        42,
        900,
        &super::super::super::tests::economy(),
    );
    let original = *city.lots.iter().find(|lot| lot.has_rear_range()).unwrap();
    let mut palette = recipes::RecipePalette::default();
    let range = palette.range().unwrap();
    let mut cache = ClearanceCache::default();
    for seed in [42, 47, 101] {
        let front_recipe = palette
            .get(
                original.archetype(),
                Some(BuildingUse::Dwelling),
                None,
                seed,
            )
            .unwrap();
        for yaw in [0.0, 0.71] {
            for passage_side in [PropertySide::Left, PropertySide::Right] {
                let lot = CityBuildingLot {
                    centre_metres: Vec2::ZERO,
                    orientation: BuildingOrientation::from_radians(yaw).unwrap(),
                    passage_side,
                    ..original
                };
                let front = front_recipe.place(lot.id, lot.centre_metres, lot.orientation);
                let world = |p| lot.orientation.local_to_world(p);
                let street_y =
                    -lot.footprint_metres.y * 0.5 - compound::COMPOUND_EDGE_MARGIN_METRES - 3.5;
                let streets = [CityStreetPatch::Corridor {
                    start_metres: world(Vec2::new(-40.0, street_y)),
                    end_metres: world(Vec2::new(40.0, street_y)),
                    half_width_metres: 3.5,
                    surface: super::super::super::CityStreetSurface::CompactedEarth,
                }];
                let (rear, property) =
                    compile(lot, &front, &front_recipe, &range, &streets, &mut cache)
                        .unwrap_or_else(|error| {
                            panic!("seed={seed}, yaw={yaw}, side={passage_side:?}: {error}")
                        });
                // Repeat the physical proof without the reusable recipe cache.
                clearance::validate(&property, &front, &front_recipe, &rear, &range).unwrap();
                let gate_local = lot
                    .orientation
                    .world_to_local(property.boundary.gate.centre_metres);
                assert!(gate_local.x * passage_side.sign() > lot.footprint_metres.x * 0.5);
                assert_eq!(property.boundary.gate.hinge, passage_side.opposite());
                assert_ne!(property.front_building_id, property.rear_building_id);
                assert!(rear.program.usage.is_none());
                let mut broken = property.clone();
                broken.access[0].end_metres = front.centre_metres;
                assert!(
                    clearance::validate(&broken, &front, &front_recipe, &rear, &range).is_err()
                );
            }
        }
    }
}
