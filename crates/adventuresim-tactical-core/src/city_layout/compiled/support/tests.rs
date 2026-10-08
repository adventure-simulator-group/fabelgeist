use super::*;
use serde_json::Value;
mod diagnostic;
mod neighboring;

#[test]
fn required_population_seed_matrix_retains_capacity_and_exact_members_on_gentle_source_relief() {
    use bevy::math::Vec3;
    let point = |x: f32, z: f32| Vec3::new(x, 20.0 + x * 0.03 + z * 0.04, z);
    let corners = [
        point(-2000.0, -2000.0),
        point(2000.0, -2000.0),
        point(2000.0, 2000.0),
        point(-2000.0, 2000.0),
    ];
    let source = GeographicSurface::from_triangles([
        [corners[0], corners[1], corners[2]],
        [corners[0], corners[2], corners[3]],
    ])
    .unwrap();
    let (_, _, mut policy) = fixture();
    policy.street_apron = StreetApronDimensions::from_metres(Vec2::new(1.0, 4.0)).unwrap();
    let cases = [900, 6500, 12000]
        .into_iter()
        .flat_map(|population| {
            [42, 47, 101]
                .map(fabelgeist_determinism::Seed::from_u64)
                .into_iter()
                .map(move |seed| (population, seed))
        })
        .chain([(30000, fabelgeist_determinism::Seed::from_u64(101))]);
    for (population, seed) in cases {
        let generated = CitySite::central_german_market_town()
            .unwrap()
            .generate(
                seed,
                adventuresim_core::settlement_property::ResidentCount::new(population),
                &super::super::super::tests::economy(),
            )
            .unwrap();
        assert_eq!(
            generated.unhoused_population,
            ResidentCount::ZERO,
            "population {population}, seed {seed}"
        );
        assert!(generated.unplaced_services.is_empty());
        assert!(generated.demand_shortfalls.is_empty());
        let layout = generated
            .compile(seed)
            .unwrap_or_else(|error| panic!("population {population}, seed {seed}: {error:?}"))
            .partition(Some(50.0))
            .unwrap();
        let original = layout.clone();
        let selected =
            SelectedCityGrounding::select(&layout, &source, policy).unwrap_or_else(|error| {
                if let CityGroundingError::Binding(binding) = &error {
                    diagnostic::save_property(population, seed, &layout, &source, binding);
                }
                panic!("population {population}, seed {seed}: {error}")
            });
        let members: Vec<_> = selected.member_support().collect();
        let surfaces = selected.support_surfaces().to_vec();
        assert_eq!(members.len(), layout.playable.len() + layout.distant.len());
        assert_eq!(
            layout, original,
            "support selection mutated the original layout"
        );
        let grounded = selected.compile().unwrap_or_else(|error| {
            diagnostic::save(population, seed, &layout, &source, &surfaces, &error);
            panic!("population {population}, seed {seed}: {error:?}")
        });
        let composed = grounded.terrain();
        assert_eq!(composed.foundations().len(), surfaces.len());
        let accepted: std::collections::BTreeMap<_, _> = grounded
            .layout()
            .playable
            .iter()
            .cloned()
            .chain(
                grounded
                    .layout()
                    .distant
                    .iter()
                    .copied()
                    .map(TacticalBuildingPlacement::from),
            )
            .map(|p| (p.id, p))
            .collect();
        let original: std::collections::BTreeMap<_, _> = layout
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
            .map(|p| (p.id, p))
            .collect();
        assert_eq!(accepted.len(), original.len());
        for member in members {
            let placement = &accepted[&member.building_id];
            let mut expected = original[&member.building_id].clone();
            expected.base_elevation_metres = member.elevation;
            assert_eq!(
                placement, &expected,
                "identity/programme/horizontal geometry changed"
            );
            for point in [
                member.contact.centre_metres(),
                member.court_threshold_metres.metres(),
            ] {
                assert!(
                    composed
                        .elevations_at(
                            crate::scene_coordinates::ScenePlanPoint::from_metres(point).unwrap()
                        )
                        .iter()
                        .any(|height| (height.metres() - member.elevation.metres()).abs()
                            <= policy.limits.contact_tolerance_metres()),
                    "population {population}, seed {seed}, member {} at {point:?}",
                    member.building_id
                );
            }
        }
        assert_eq!(grounded.layout().compounds, layout.compounds);
        assert_eq!(
            grounded.layout().single_properties,
            layout.single_properties
        );
        assert_eq!(grounded.layout().gardens, layout.gardens);
        assert_eq!(grounded.layout().businesses, layout.businesses);
        for surface in surfaces {
            assert!(surface.mesh().maximum_grade() <= 0.65);
        }
    }
}

fn fixture() -> (CitySceneLayout, GeographicSurface, CompoundGradingPolicy) {
    let value: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/goslar-1238.json"
    )))
    .unwrap();
    let mut layout = CitySceneLayout::default();
    layout
        .compounds
        .push(serde_json::from_value(value["compound"].clone()).unwrap());
    for key in ["front_distant_placement", "rear_distant_placement"] {
        layout
            .distant
            .push(serde_json::from_value(value[key].clone()).unwrap());
    }
    let triangles: Vec<[bevy::math::Vec3; 3]> =
        serde_json::from_value(value["geographic_triangles"].clone()).unwrap();
    let policy = CompoundGradingPolicy {
        limits: SupportLimits::new(
            crate::city_layout::grounding::SupportGrade::from_ratio(0.65).unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.001)
                .unwrap(),
        ),
        stairs: CourtStairLimits::new(
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.19)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.25)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.0)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.05)
                .unwrap(),
            adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
                .unwrap(),
        ),
        embedment: FoundationEmbedment::from_metres(0.2).unwrap(),
        street_apron: StreetApronDimensions::from_metres(Vec2::new(1.0, 2.0)).unwrap(),
    };
    (
        layout,
        GeographicSurface::from_triangles(triangles).unwrap(),
        policy,
    )
}

#[test]
fn production_programmes_bind_exact_goslar_support_without_relocating_or_mutating_the_layout() {
    let (layout, source, policy) = fixture();
    let original = layout.clone();
    let plans = layout.plan_compound_support(&source, policy).unwrap();
    assert_eq!(layout, original);
    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    assert_eq!(plan.property_id(), CityPropertyId(1238));
    assert_eq!(plan.reservation(), layout.compounds[0].plot);
    assert_eq!(
        plan.member_support().map(|m| m.building_id),
        [1238, 17622].map(crate::scene_input::SceneBuildingId)
    );
    assert!((plan.member_support()[0].elevation.metres() - 21.042906).abs() < 0.001);
    assert!((plan.member_support()[1].elevation.metres() - 19.056694).abs() < 0.001);
    plan.foundations(&source, policy.embedment).unwrap();
}

#[test]
fn promotion_retains_the_same_property_support_and_source_iteration_does_not_select_other_floors() {
    let (mut layout, source, policy) = fixture();
    let initial = layout.plan_compound_support(&source, policy).unwrap();
    layout.playable = layout
        .distant
        .drain(..)
        .map(TacticalBuildingPlacement::from)
        .collect();
    layout.playable.reverse();
    let promoted = layout.plan_compound_support(&source, policy).unwrap();
    assert_eq!(initial[0].mesh(), promoted[0].mesh());
    assert_eq!(initial[0].member_support(), promoted[0].member_support());
}

#[test]
fn a_missing_range_reports_its_authoritative_id_without_substituting_another_member() {
    let (mut layout, source, policy) = fixture();
    layout.distant.retain(|b| b.id != (17622).into());
    let error = layout.plan_compound_support(&source, policy).unwrap_err();
    assert!(matches!(
        error,
        CitySupportError::MissingBuilding {
            property: CityPropertyId(1238),
            building: crate::scene_input::SceneBuildingId(17622)
        }
    ));
    assert_eq!(layout.distant.len(), 1);
}

#[test]
fn both_public_planners_reject_duplicate_physical_placements_before_collection() {
    let (mut layout, source, policy) = fixture();
    let duplicate = layout.distant[0];
    layout.distant.push(duplicate);
    assert!(
        matches!(layout.plan_compound_support(&source, policy), Err(CitySupportError::DuplicateBuilding { building }) if building == duplicate.id)
    );
    let single_policy = SinglePropertyGradingPolicy {
        limits: policy.limits,
        stairs: policy.stairs,
        embedment: policy.embedment,
        doorway_apron: policy.street_apron,
    };
    assert!(
        matches!(layout.plan_single_property_support(&source, single_policy), Err(CitySupportError::DuplicateBuilding { building }) if building == duplicate.id)
    );
}
