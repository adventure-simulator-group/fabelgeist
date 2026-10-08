//! Admission and diagnostics at the public geographic support boundary.
use adventuresim_tactical_core::{
    city_layout::{
        CityPlotBounds, CityPropertyId, CitySingleProperty, SinglePropertyGradingPolicy,
        StreetApronDimensions, grounding::*,
    },
    scene_input::BuildingOrientation,
};
use bevy::math::{Vec2, Vec3};

#[test]
fn invalid_policies_cannot_be_admitted_by_decoding() {
    assert!(FoundationEmbedment::from_metres(-1.0).is_none());
    assert!(serde_json::from_str::<FoundationEmbedment>("-1").is_err());
    let negative = postcard::to_allocvec(&-1.0_f32).unwrap();
    assert!(postcard::from_bytes::<FoundationEmbedment>(&negative).is_err());
    assert!(serde_json::from_str::<SupportLimits>(r#"{"maximum_grade":-1,"maximum_displacement_metres":6,"contact_tolerance_metres":0.001}"#).is_err());
    assert!(serde_json::from_str::<CourtStairLimits>(r#"{"maximum_riser_metres":-1,"minimum_going_metres":0.25,"minimum_clear_width_metres":1,"minimum_floor_landing_run_metres":1.05,"minimum_court_landing_run_metres":0.5}"#).is_err());
}

#[test]
fn a_missing_required_entrance_reports_one_missing_binding() {
    let source = GeographicSurface::from_triangles([
        [
            Vec3::new(-30.0, 0.0, -30.0),
            Vec3::new(30.0, 0.0, -30.0),
            Vec3::new(30.0, 0.0, 30.0),
        ],
        [
            Vec3::new(-30.0, 0.0, -30.0),
            Vec3::new(30.0, 0.0, 30.0),
            Vec3::new(-30.0, 0.0, 30.0),
        ],
    ])
    .unwrap();
    let plot = CityPlotBounds::new(
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(Vec2::ZERO)
            .unwrap(),
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(Vec2::new(
            12.0, 24.0,
        ))
        .unwrap(),
        BuildingOrientation::IDENTITY,
    )
    .unwrap();
    let bearing = (plot)
        .resized(
            adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                Vec2::new(8.0, 12.0),
            )
            .unwrap(),
        )
        .unwrap();
    let error = SingleBuildingSupportRequest {
        property: CitySingleProperty {
            id: CityPropertyId(7),
            building_id: adventuresim_tactical_core::scene_input::SceneBuildingId(7),
            plot,
        },
        bearing,
        bearing_region: FloorRegion::from_scene(
            adventuresim_tactical_core::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                bearing
                    .corners()
                    .into_iter()
                    .map(|p| {
                        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                            p,
                        )
                        .unwrap()
                    })
                    .collect(),
            )
            .unwrap(),
        )
        .unwrap(),
        thresholds: &[],
        geographic: &source,
        streets: &[],
        policy: SinglePropertyGradingPolicy {
            limits: SupportLimits::new(
                adventuresim_tactical_core::city_layout::grounding::SupportGrade::from_ratio(0.65)
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(6.0)
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    0.001,
                )
                .unwrap(),
            ),
            stairs: CourtStairLimits::new(
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    0.19,
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    0.25,
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.0)
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    1.05,
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(0.5)
                    .unwrap(),
            ),
            embedment: FoundationEmbedment::from_metres(0.2).unwrap(),
            doorway_apron: StreetApronDimensions::from_metres(Vec2::new(1.0, 4.0)).unwrap(),
        },
    }
    .select()
    .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::ThresholdBinding);
    assert_eq!(error.violation.actual_value(), 0.0);
    assert_eq!(error.violation.limit_value(), 1.0);
    assert_eq!(error.violation.discrepancy_value(), 1.0);
}

#[test]
fn checked_policies_roundtrip_and_reflection_cannot_edit_their_fields() {
    use bevy::reflect::{PartialReflect, ReflectMut};
    let policy =
        adventuresim_tactical_core::city_layout::CompoundGradingPolicy::bounded_settlement();
    let json = serde_json::to_value(policy.limits).unwrap();
    for field in [
        "maximum_grade",
        "maximum_displacement_metres",
        "contact_tolerance_metres",
    ] {
        for invalid in [-1.0, 0.0] {
            let mut changed = json.clone();
            changed[field] = serde_json::json!(invalid);
            assert!(serde_json::from_value::<SupportLimits>(changed).is_err());
        }
    }
    let stair_json = serde_json::to_value(policy.stairs).unwrap();
    for field in [
        "maximum_riser_metres",
        "minimum_going_metres",
        "minimum_clear_width_metres",
        "minimum_floor_landing_run_metres",
        "minimum_court_landing_run_metres",
    ] {
        for invalid in [-1.0, 0.0] {
            let mut changed = stair_json.clone();
            changed[field] = serde_json::json!(invalid);
            assert!(serde_json::from_value::<CourtStairLimits>(changed).is_err());
        }
    }
    #[derive(serde::Serialize)]
    struct NativeLimits {
        maximum_grade: f32,
        maximum_displacement_metres: f32,
        contact_tolerance_metres: f32,
    }
    for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for slot in 0..3 {
            let mut values = [0.65, 6.0, 0.001];
            values[slot] = invalid;
            let wire = NativeLimits {
                maximum_grade: values[0],
                maximum_displacement_metres: values[1],
                contact_tolerance_metres: values[2],
            };
            assert!(
                postcard::from_bytes::<SupportLimits>(&postcard::to_allocvec(&wire).unwrap())
                    .is_err()
            );
        }
        assert!(SupportGrade::from_ratio(invalid).is_none());
        assert!(FoundationEmbedment::from_metres(invalid).is_none());
    }
    let mut limits = policy.limits;
    let mut stairs = policy.stairs;
    let mut embedment = policy.embedment;
    assert!(matches!(limits.reflect_mut(), ReflectMut::Opaque(_)));
    assert!(matches!(stairs.reflect_mut(), ReflectMut::Opaque(_)));
    assert!(matches!(embedment.reflect_mut(), ReflectMut::Opaque(_)));
    assert_eq!(
        serde_json::from_value::<SupportLimits>(json).unwrap(),
        limits
    );
    assert_eq!(
        postcard::from_bytes::<SupportLimits>(&postcard::to_allocvec(&limits).unwrap()).unwrap(),
        limits
    );
    assert_eq!(
        postcard::from_bytes::<CourtStairLimits>(&postcard::to_allocvec(&stairs).unwrap()).unwrap(),
        stairs
    );
    assert_eq!(
        postcard::from_bytes::<FoundationEmbedment>(&postcard::to_allocvec(&embedment).unwrap())
            .unwrap(),
        embedment
    );
}

fn prism(
    width: f32,
) -> [adventuresim_building_generator::spatial_geometry::Position<
    adventuresim_tactical_core::scene_coordinates::Scene,
>; 6] {
    use adventuresim_building_generator::spatial_geometry::Position;
    [
        Vec3::new(0.0, -2.0, 0.0),
        Vec3::new(width, -2.0, 0.0),
        Vec3::new(0.0, -2.0, width),
        Vec3::new(0.0, -2.2, 0.0),
        Vec3::new(width, -2.2, 0.0),
        Vec3::new(0.0, -2.2, width),
    ]
    .map(|p| Position::from_metres(p).unwrap())
}

#[test]
fn members_preserve_order_and_geometry_admission_keeps_thin_contact_cells() {
    use adventuresim_tactical_core::scene_input::SceneBuildingId;
    let members = PropertyMembers::new(vec![SceneBuildingId(9), SceneBuildingId(7)]).unwrap();
    assert_eq!(members.ids(), &[SceneBuildingId(9), SceneBuildingId(7)]);
    assert_eq!(
        PropertyMembers::new(vec![SceneBuildingId(9), SceneBuildingId(9)]).unwrap_err(),
        PropertyMemberIssue::Duplicate {
            building: SceneBuildingId(9),
            first: 0,
            index: 1
        }
    );
    assert!(serde_json::from_str::<PropertyMembers>("[9,9]").is_err());
    assert!(
        postcard::from_bytes::<PropertyMembers>(&postcard::to_allocvec(&vec![9_u64, 9]).unwrap())
            .is_err()
    );
    for width in [1.0, 0.000_000_01, 0.0] {
        let mesh = PropertyFoundationMesh::from_prisms(
            CityPropertyId(7),
            members.clone(),
            vec![prism(width)],
            Vec::new(),
        )
        .unwrap();
        assert_eq!(
            serde_json::from_value::<PropertyFoundationMesh>(serde_json::to_value(&mesh).unwrap())
                .unwrap(),
            mesh
        );
        assert_eq!(
            postcard::from_bytes::<PropertyFoundationMesh>(&postcard::to_allocvec(&mesh).unwrap())
                .unwrap(),
            mesh
        );
        let mut malformed = serde_json::to_value(&mesh).unwrap();
        malformed["solid_triangles"][0][0] = serde_json::json!(9999);
        assert!(serde_json::from_value::<PropertyFoundationMesh>(malformed).is_err());
    }
}

#[test]
fn finite_vertical_queries_preserve_negative_levels_and_explicit_ceilings() {
    use adventuresim_building_generator::spatial_geometry::PositiveLength;
    use adventuresim_tactical_core::scene_coordinates::ScenePlanPoint;
    let mesh = PropertyFoundationMesh::from_prisms(
        CityPropertyId(7),
        PropertyMembers::new(vec![7.into()]).unwrap(),
        vec![prism(1.0)],
        Vec::new(),
    )
    .unwrap();
    let surface = BoundedSettlementTerrain::from_foundations(
        vec![mesh],
        Vec::new(),
        PositiveLength::from_metres(0.001).unwrap(),
    )
    .unwrap();
    let point = ScenePlanPoint::from_metres(Vec2::splat(0.2)).unwrap();
    let unbounded = surface
        .surface_below(SupportQuery::unbounded(point))
        .unwrap();
    assert_eq!(unbounded.elevation.metres(), -2.0);
    assert_eq!(surface.highest_surface_at(point), Some(unbounded));
    assert_eq!(
        surface.surface_below(SupportQuery::bounded(
            point,
            SupportElevation::from_metres(-1.0).unwrap()
        )),
        Some(unbounded)
    );
    assert!(
        surface
            .surface_below(SupportQuery::bounded(
                point,
                SupportElevation::from_metres(-3.0).unwrap()
            ))
            .is_none()
    );
    assert!(SupportQuery::try_from(Vec3::new(f32::NAN, 0.0, 0.0)).is_err());
    assert!(SupportQuery::try_from(Vec3::new(0.0, f32::INFINITY, 0.0)).is_err());
    let mut malformed = serde_json::to_value(&surface).unwrap();
    malformed["query"][0]["children"] = serde_json::json!({"Branch":{"left":0,"right":0}});
    assert!(serde_json::from_value::<BoundedSettlementTerrain>(malformed).is_err());
    let mut malformed = serde_json::to_value(&surface).unwrap();
    malformed["query"][0]["children"] =
        serde_json::json!({"Leaf":[{"Foundation":{"owner":9,"triangle":0}}]});
    assert!(serde_json::from_value::<BoundedSettlementTerrain>(malformed).is_err());
    assert_eq!(
        postcard::from_bytes::<BoundedSettlementTerrain>(&postcard::to_allocvec(&surface).unwrap())
            .unwrap(),
        surface
    );
}

#[test]
fn decoded_support_mesh_rejects_empty_topology_before_physics() {
    use adventuresim_tactical_core::scene_input::SceneBuildingId;
    let invalid = serde_json::json!({
        "property_id": 1, "member_building_ids": [1],
        "positions": [], "support_triangles": [], "retaining_triangles": [],
        "contact_tolerance_metres": 0.001
    });
    assert!(serde_json::from_value::<PropertySupportMesh>(invalid).is_err());
    let empty = postcard::to_allocvec(&(
        CityPropertyId(1),
        vec![SceneBuildingId(1)],
        Vec::<Vec3>::new(),
        Vec::<[u32; 3]>::new(),
        Vec::<[u32; 3]>::new(),
        0.001_f32,
    ))
    .unwrap();
    assert!(postcard::from_bytes::<PropertySupportMesh>(&empty).is_err());
    let valid = serde_json::json!({
        "property_id": 1, "member_building_ids": [1],
        "positions": [Vec3::ZERO, Vec3::X, Vec3::Z],
        "support_triangles": [[0, 2, 1]], "retaining_triangles": [],
        "contact_tolerance_metres": 0.001
    });
    let mesh = serde_json::from_value::<PropertySupportMesh>(valid).unwrap();
    assert!(mesh.collider().is_ok());
    let bytes = postcard::to_allocvec(&mesh).unwrap();
    assert_eq!(
        postcard::from_bytes::<PropertySupportMesh>(&bytes).unwrap(),
        mesh
    );
}

#[test]
fn foundation_physics_preserves_windings_thin_prisms_and_contact_cells() {
    use bevy::math::Quat;
    for width in [1.0_f32, 0.000_000_01, 0.0] {
        for reversed in [false, true] {
            let mut points = prism(width);
            if reversed {
                points.swap(1, 2);
                points.swap(4, 5);
            }
            let mesh = PropertyFoundationMesh::from_prisms(
                CityPropertyId(7),
                PropertyMembers::new(vec![7.into()]).unwrap(),
                vec![points],
                Vec::new(),
            )
            .unwrap();
            let collision = mesh.collider().unwrap();
            if width == 0.0 {
                assert!(matches!(collision, SupportCollision::ContactOnly));
            } else {
                let collider = collision.into_solid().unwrap();
                assert!(collider.contains_point(
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    Vec3::new(width * 0.2, -2.1, width * 0.2)
                ));
                assert!(!collider.contains_point(
                    Vec3::ZERO,
                    Quat::IDENTITY,
                    Vec3::new(width * 0.2, -1.0, width * 0.2)
                ));
                let hit = collider
                    .cast_ray(
                        Vec3::ZERO,
                        Quat::IDENTITY,
                        Vec3::new(width * 0.2, -1.0, width * 0.2),
                        Vec3::NEG_Y,
                        2.0,
                        false,
                    )
                    .unwrap();
                assert!((hit.0 - 1.0).abs() < 0.000_01);
                assert!(hit.1.y > 0.99);
            }
        }
    }
}

#[test]
fn terrain_policy_decoding_preserves_zero_and_rejects_invalid_leaves() {
    use adventuresim_tactical_core::{
        prelude::{CollarWidthVariation, RuptureWander},
        scene::TerrainGradeLimit,
    };
    use bevy::reflect::{PartialReflect, ReflectMut};
    for value in [0.0, 0.65] {
        let mut grade = TerrainGradeLimit::from_ratio(value).unwrap();
        assert_eq!(
            serde_json::from_value::<TerrainGradeLimit>(serde_json::json!(value)).unwrap(),
            grade
        );
        assert_eq!(
            postcard::from_bytes::<TerrainGradeLimit>(&postcard::to_allocvec(&grade).unwrap())
                .unwrap(),
            grade
        );
        assert!(matches!(grade.reflect_mut(), ReflectMut::Opaque(_)));
        let mut wander = RuptureWander::from_metres(value).unwrap();
        assert_eq!(
            postcard::from_bytes::<RuptureWander>(&postcard::to_allocvec(&wander).unwrap())
                .unwrap(),
            wander
        );
        assert_eq!(
            serde_json::from_value::<RuptureWander>(serde_json::json!(value)).unwrap(),
            wander
        );
        assert!(matches!(wander.reflect_mut(), ReflectMut::Opaque(_)));
    }
    for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(TerrainGradeLimit::from_ratio(value).is_none());
        assert!(RuptureWander::from_metres(value).is_none());
        let bytes = postcard::to_allocvec(&value).unwrap();
        assert!(postcard::from_bytes::<TerrainGradeLimit>(&bytes).is_err());
        assert!(postcard::from_bytes::<RuptureWander>(&bytes).is_err());
    }
    assert!(serde_json::from_str::<TerrainGradeLimit>("-1").is_err());
    assert!(serde_json::from_str::<RuptureWander>("-1").is_err());
    for value in [0_u16, 5000] {
        let mut variation = CollarWidthVariation::from_basis_points(value).unwrap();
        assert_eq!(
            postcard::from_bytes::<CollarWidthVariation>(
                &postcard::to_allocvec(&variation).unwrap()
            )
            .unwrap(),
            variation
        );
        assert_eq!(
            serde_json::from_value::<CollarWidthVariation>(serde_json::json!(value)).unwrap(),
            variation
        );
        assert!(matches!(variation.reflect_mut(), ReflectMut::Opaque(_)));
    }
    assert!(CollarWidthVariation::from_basis_points(5001).is_none());
    assert!(serde_json::from_str::<CollarWidthVariation>("5001").is_err());
    assert!(
        postcard::from_bytes::<CollarWidthVariation>(&postcard::to_allocvec(&5001_u16).unwrap())
            .is_err()
    );
}

#[test]
fn finite_extreme_support_endpoints_never_publish_infinite_elevation() {
    use adventuresim_tactical_core::scene_coordinates::ScenePlanPoint;
    let value = serde_json::json!({
        "property_id": 1, "member_building_ids": [1],
        "positions": [Vec3::new(0.0, -f32::MAX, 0.0), Vec3::new(1.0, f32::MAX, 0.0), Vec3::new(0.0, -f32::MAX, 1.0)],
        "support_triangles": [[0, 1, 2]], "retaining_triangles": [],
        "contact_tolerance_metres": 0.001
    });
    let mesh = serde_json::from_value::<PropertySupportMesh>(value).unwrap();
    let heights = mesh.elevations_at(ScenePlanPoint::from_metres(Vec2::new(0.5, 0.25)).unwrap());
    let heights: Vec<_> = heights.iter().collect();
    assert_eq!(heights.len(), 1);
    assert_eq!(heights[0].metres(), 0.0);
}

#[test]
fn zero_garden_identities_fail_construction_and_both_decoders() {
    use adventuresim_tactical_core::scene_input::{
        GardenSupportError, SceneGarden, TacticalSceneInput,
    };
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../assets/tactical-scenes/garden-review.json"
    ))
    .unwrap();
    let terrain = input.generate().unwrap().terrain;
    let accepted = SceneGarden::project(input.gardens[0].clone(), &terrain).unwrap();
    #[derive(serde::Serialize)]
    struct GardenWire {
        garden: adventuresim_tactical_core::city_layout::CityGarden,
        plant_support: Vec<adventuresim_tactical_core::scene_input::GardenPlantSupport>,
    }
    for case in 0..3 {
        let mut garden = accepted.garden().clone();
        match case {
            0 => garden.owner.0 = 0,
            1 => garden.front_building_id.0 = 0,
            _ => garden.plants[0].id.0 = 0,
        }
        let roots = accepted.plant_support().to_vec();
        assert!(matches!(
            SceneGarden::from_support(garden.clone(), roots.clone()),
            Err(GardenSupportError::Identity { .. })
        ));
        let wire = GardenWire {
            garden,
            plant_support: roots,
        };
        assert!(
            serde_json::from_value::<SceneGarden>(serde_json::to_value(&wire).unwrap()).is_err()
        );
        assert!(
            postcard::from_bytes::<SceneGarden>(&postcard::to_allocvec(&wire).unwrap()).is_err()
        );
    }
}
