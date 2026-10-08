use super::*;
fn region(x: f32, y: f32, width: f32, height: f32) -> FloorRegion {
    FloorRegion::from_scene(
        ScenePlanPolygon::from_ordered_vertices(
            [
                Vec2::new(x, y),
                Vec2::new(x + width, y),
                Vec2::new(x + width, y + height),
                Vec2::new(x, y + height),
            ]
            .map(|p| ScenePlanPoint::try_from(p).unwrap())
            .to_vec(),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn signed_zero_shared_edges_cancel_without_changing_vertices() {
    let lower = region(-1.0, -1.0, 1.0, 1.0);
    let upper = FloorRegion::from_scene(
        ScenePlanPolygon::from_ordered_vertices(
            [
                Vec2::new(-1.0, -0.0),
                Vec2::new(0.0, -0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(-1.0, 1.0),
            ]
            .map(|p| ScenePlanPoint::try_from(p).unwrap())
            .to_vec(),
        )
        .unwrap(),
    )
    .unwrap();
    let accepted =
        FloorBearing::admit(1.into(), vec![lower, upper], SupportElevation::ZERO).unwrap();
    assert_eq!(accepted.exterior_perimeter(), 6.0);
    assert_eq!(
        accepted.regions()[1].outline().vertices()[0]
            .metres()
            .y
            .to_bits(),
        (-0.0_f32).to_bits()
    );
}
#[test]
fn partition_admission_rejects_empty_duplicate_and_partial_shared_edges() {
    let admit = |regions| {
        FloorBearing::admit(
            1.into(),
            regions,
            SupportElevation::from_metres(0.0).unwrap(),
        )
    };
    assert!(admit(Vec::new()).is_err());
    assert!(matches!(
        FloorBearing::admit(
            0.into(),
            vec![region(0.0, 0.0, 1.0, 1.0)],
            SupportElevation::from_metres(0.0).unwrap()
        ),
        Err(FloorBearingConstructionError::InvalidBuilding(_))
    ));
    let tiny = region(0.0, 0.0, 1e-8, 1e-8);
    assert!(admit(vec![tiny.clone(), tiny]).is_err());
    assert!(
        admit(vec![
            region(0.0, 0.0, 1.0, 2.0),
            region(1.0, 0.0, 1.0, 1.0),
            region(1.0, 1.0, 1.0, 1.0)
        ])
        .is_err()
    );
    let accepted = admit(vec![
        region(0.0, 0.0, 1.0, 1.0),
        region(1.0, 0.0, 1.0, 1.0),
        region(0.0, 1.0, 1.0, 1.0),
        region(1.0, 1.0, 1.0, 1.0),
    ])
    .unwrap();
    assert_eq!(accepted.exterior_perimeter(), 8.0);
    assert_eq!(
        postcard::from_bytes::<FloorBearing>(&postcard::to_allocvec(&accepted).unwrap()).unwrap(),
        accepted
    );
    let mut json = serde_json::to_value(&accepted).unwrap();
    json["regions"] = serde_json::json!([]);
    assert!(serde_json::from_value::<FloorBearing>(json).is_err());
}

#[test]
fn rounded_architectural_edges_keep_union_coverage_without_masking_missing_faces() {
    use crate::{scene_coordinates::ArchitecturalPlanProjection, scene_input::BuildingOrientation};
    use adventuresim_building_generator::plan_geometry::{ArchitecturalPlanPoint, PlanPolygon};
    use adventuresim_building_generator::spatial_geometry::PositiveLength;
    // Frozen Kassel member 18 has an authored collinear edge that gains a tiny
    // signed bend after the established f32 rotation and translation.
    let vertices = [
        [0.120000005, 17.92],
        [0.14999962, 0.14999962],
        [6.0, -0.108],
        [9.0, -0.108],
        [13.35, 0.14999962],
        [13.379999, 1.42],
        [13.379999, 18.08],
        [13.35, 19.35],
        [7.58, 19.38],
        [3.08, 19.380001],
        [2.92, 19.380001],
        [0.14999962, 19.35],
        [0.120000005, 18.08],
    ];
    let source = PlanPolygon::from_ordered_vertices(
        vertices
            .map(|p| ArchitecturalPlanPoint::from_metres(Vec2::from_array(p)).unwrap())
            .to_vec(),
    )
    .unwrap();
    let projection = ArchitecturalPlanProjection {
        centre: ScenePlanPoint::from_metres(Vec2::new(-22.180737, -21.00796)).unwrap(),
        origin: ArchitecturalPlanPoint::from_metres(Vec2::new(6.75, 9.75)).unwrap(),
        orientation: serde_json::from_value::<BuildingOrientation>(serde_json::json!(2.9441972))
            .unwrap(),
    };
    let region = FloorRegion::from_architectural(&source, projection).unwrap();
    assert!(ScenePlanPolygon::from_ordered_vertices(region.outline().vertices().to_vec()).is_err());
    let elevation = SupportElevation::from_metres(5.0042872).unwrap();
    let bearing = FloorBearing::admit(18.into(), vec![region.clone()], elevation).unwrap();
    let plot = CityPlotBounds::new(
        projection.centre,
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
            Vec2::splat(40.0),
        )
        .unwrap(),
        BuildingOrientation::IDENTITY,
    )
    .unwrap();
    let mut mesh = PropertySupportMesh {
        property_id: CityPropertyId(18),
        member_building_ids: vec![18.into()],
        positions: Vec::new(),
        support_triangles: Vec::new(),
        retaining_triangles: Vec::new(),
        contact_tolerance_metres: 0.001,
    };
    mesh.convex_floor(region.outline(), elevation).unwrap();
    let mut surface = PropertySupportSurface {
        mesh,
        floor_bearings: vec![bearing],
        regions: vec![plot],
        clipping_outlines: vec![
            region
                .outline()
                .vertices()
                .iter()
                .map(|p| p.metres().as_dvec2())
                .collect(),
        ],
        limits: SupportLimits::new(
            SupportGrade::from_ratio(0.65).unwrap(),
            PositiveLength::from_metres(6.0).unwrap(),
            PositiveLength::from_metres(0.001).unwrap(),
        ),
        treatment: SupportGradingAttempt::SingleBuildingFloorAndEntrances,
    };
    surface.validate_floors().unwrap();
    let encoded = serde_json::to_vec(&surface).unwrap();
    let restored: PropertySupportSurface = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(restored, surface);
    let replicated: PropertySupportSurface =
        postcard::from_bytes(&postcard::to_allocvec(&surface).unwrap()).unwrap();
    assert_eq!(replicated, surface);
    surface.mesh.support_triangles.remove(0);
    surface
        .mesh
        .support_triangles
        .push(surface.mesh.support_triangles[0]);
    assert!(matches!(
        surface.validate_floors(),
        Err(SupportSurfaceIssue::Floor {
            issue: FloorBearingIssue::Coverage { .. },
            ..
        })
    ));
}
