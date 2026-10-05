//! Frozen positive fixture: rectangular envelopes overlap but bearings do not.
use super::*;
#[test]
fn population_6500_seed_42_neighboring_members_keep_exact_support_without_empty_corner_ownership() {
    let value: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/neighboring-property-support.json"
    )))
    .unwrap();
    // The frozen source and IDs reproduce the original neighbouring case.
    // Reservations and member transforms come from the general geometry-aware
    // packing calculation, rather than obsolete pre-packing rectangles.
    let compiled = CitySite::central_german_market_town()
        .generate(42, 6500, &crate::city_layout::tests::economy())
        .compile(42)
        .unwrap();
    let mut layout = compiled.partition(None).unwrap();
    layout.playable.extend(
        layout
            .distant
            .drain(..)
            .filter(|b| [519, 520, 16903].contains(&b.id))
            .map(TacticalBuildingPlacement::from),
    );
    layout
        .playable
        .retain(|b| [519, 520, 16903].contains(&b.id));
    layout.compounds.retain(|p| p.id == CityPropertyId(519));
    layout
        .single_properties
        .retain(|p| p.id == CityPropertyId(520));
    assert_eq!(layout.playable.len(), 3);
    let source = GeographicSurface::from_triangles(
        serde_json::from_value::<Vec<[bevy::math::Vec3; 3]>>(value["source_triangles"].clone())
            .unwrap(),
    )
    .unwrap();
    let original = layout.clone();
    let (_, _, mut policy) = fixture();
    policy.street_apron = StreetApronDimensions::from_metres(Vec2::new(1.0, 4.0)).unwrap();
    let single_policy = SinglePropertyGradingPolicy {
        limits: policy.limits,
        stairs: policy.stairs,
        embedment: policy.embedment,
        doorway_apron: policy.street_apron,
    };
    let compounds = layout.plan_compound_support(&source, policy).unwrap();
    let singles = layout
        .plan_single_property_support(&source, single_policy)
        .unwrap();
    assert_eq!(layout, original);
    assert_eq!(singles[0].surface.member_building_ids(), [520]);
    assert_eq!(
        compounds[0].member_support().map(|m| m.building_id),
        [519, 16903]
    );
    let surfaces: Vec<_> = compounds
        .iter()
        .map(|plan| plan.support_surface().unwrap())
        .chain(singles.iter().map(|p| p.surface.clone()))
        .collect();
    let terrain = BoundedSettlementTerrain::compile(&surfaces, &source, policy.embedment).unwrap();
    let floor = &singles[0];
    let recipe = crate::scene_input::GeneratedBuildingRecipe::generate(
        layout
            .playable
            .iter()
            .find(|b| b.id == 520)
            .unwrap()
            .program
            .clone(),
    )
    .unwrap();
    let placement = layout.playable.iter().find(|b| b.id == 520).unwrap();
    use bevy::math::Vec3Swizzles;
    for point in recipe
        .collision
        .cuboids
        .iter()
        .flat_map(|c| c.ground_contact().unwrap().points().collect::<Vec<_>>())
    {
        let world = placement.centre_metres
            + placement
                .orientation
                .local_to_world(point.metres() - recipe.collision.bounds.centre().xz());
        assert!(
            terrain
                .elevations_at(
                    crate::scene_coordinates::ScenePlanPoint::from_metres(world).unwrap()
                )
                .iter()
                .any(|height| (height.metres() - floor.floor.elevation.metres()).abs() < 0.001),
            "contact {world:?}, floor {:?}, support {:?}",
            floor.floor.elevation,
            terrain.elevations_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(world).unwrap()
            )
        );
    }
    let empty_corner = Vec2::new(-105.0, -297.9);
    assert!(!floor.surface.contains(empty_corner));
    assert!(!surfaces.iter().any(|s| s.contains(empty_corner)));
    {
        assert!(
            (terrain
                .highest_surface_at(
                    crate::scene_coordinates::ScenePlanPoint::from_metres(empty_corner).unwrap()
                )
                .unwrap()
                .elevation
                .metres()
                - source.elevation_at(empty_corner).unwrap().metres())
            .abs()
                < policy.limits.contact_tolerance_metres()
        );
    }
    layout.playable.reverse();
    let promoted = layout
        .plan_single_property_support(&source, single_policy)
        .unwrap();
    assert_eq!(floor.surface.mesh(), promoted[0].surface.mesh());
    let mut reversed = surfaces;
    reversed.reverse();
    assert_eq!(
        terrain,
        BoundedSettlementTerrain::compile(&reversed, &source, policy.embedment).unwrap()
    );
}
