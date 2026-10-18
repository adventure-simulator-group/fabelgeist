use super::*;
use adventuresim_building_generator::{BuildingEntranceId, OpeningAssemblyId};

fn source(grade: f32) -> GeographicSurface {
    let points = [
        Vec2::splat(-30.0),
        Vec2::new(30.0, -30.0),
        Vec2::splat(30.0),
        Vec2::new(-30.0, 30.0),
    ]
    .map(|p| Vec3::new(p.x, p.y * grade, p.y));
    GeographicSurface::from_triangles([
        [points[0], points[1], points[2]],
        [points[0], points[2], points[3]],
    ])
    .unwrap()
}
fn policy() -> SinglePropertyGradingPolicy {
    SinglePropertyGradingPolicy {
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
        doorway_apron: crate::city_layout::StreetApronDimensions::from_metres(Vec2::new(1.0, 4.0))
            .unwrap(),
    }
}
fn request<'a>(
    source: &'a GeographicSurface,
    doors: &'a [DoorwaySupportBinding],
) -> SingleBuildingSupportRequest<'a> {
    SingleBuildingSupportRequest {
        property: CitySingleProperty {
            id: CityPropertyId(7),
            building_id: (7).into(),
            plot: CityPlotBounds::new(
                crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::ZERO).unwrap(),
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    Vec2::new(12.0, 24.0),
                )
                .unwrap(),
                BuildingOrientation::IDENTITY,
            )
            .unwrap(),
        },
        bearing: CityPlotBounds::new(
            crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::ZERO).unwrap(),
            adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                Vec2::new(8.0, 12.0),
            )
            .unwrap(),
            BuildingOrientation::IDENTITY,
        )
        .unwrap(),
        bearing_region: FloorRegion::from_scene(
            CityPlotBounds::new(
                crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::ZERO).unwrap(),
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    Vec2::new(8.0, 12.0),
                )
                .unwrap(),
                BuildingOrientation::IDENTITY,
            )
            .unwrap()
            .plan_polygon()
            .unwrap(),
        )
        .unwrap(),
        thresholds: doors,
        geographic: source,
        streets: &[],
        policy: policy(),
    }
}
fn doors() -> [DoorwaySupportBinding; 2] {
    [
        DoorwaySupportBinding {
            entrance: BuildingEntranceId::Opening(OpeningAssemblyId(8)),
            support: adventuresim_building_generator::BuildingEntranceSupport::ArchitecturalFloor,
            threshold_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(
                0.0, -5.9,
            ))
            .unwrap(),
            outward:
                adventuresim_building_generator::spatial_geometry::PlanDirection::from_normalized(
                    -Vec2::Y,
                )
                .unwrap(),
        },
        DoorwaySupportBinding {
            entrance: BuildingEntranceId::Opening(OpeningAssemblyId(9)),
            support: adventuresim_building_generator::BuildingEntranceSupport::ArchitecturalFloor,
            threshold_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(
                0.0, 5.9,
            ))
            .unwrap(),
            outward:
                adventuresim_building_generator::spatial_geometry::PlanDirection::from_normalized(
                    Vec2::Y,
                )
                .unwrap(),
        },
    ]
}
#[test]
fn single_property_supports_all_doors_without_levelling_its_court_or_garden() {
    let source = source(0.1);
    let doors = doors();
    let plan = request(&source, &doors).select().unwrap();
    assert_eq!(
        plan.surface.member_building_ids(),
        [7].map(crate::scene_input::SceneBuildingId)
    );
    let terrain = BoundedSettlementTerrain::compile(
        std::slice::from_ref(&plan.surface),
        &source,
        policy().embedment,
    )
    .unwrap();
    assert!(
        (terrain
            .highest_surface_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(Vec2::ZERO).unwrap()
            )
            .unwrap()
            .elevation
            .metres()
            - plan.floor.elevation.metres())
        .abs()
            < 0.001
    );
    let garden = Vec2::new(4.0, 11.0);
    assert!(
        !plan
            .surface
            .contains(crate::scene_coordinates::ScenePlanPoint::try_from(garden).unwrap())
    );
    assert_eq!(
        terrain
            .highest_surface_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(garden).unwrap()
            )
            .unwrap()
            .elevation
            .metres(),
        source
            .elevation_at(crate::scene_coordinates::ScenePlanPoint::try_from(garden).unwrap())
            .unwrap()
            .metres()
    );
    for door in doors {
        assert!(
            (terrain
                .highest_surface_at(
                    crate::scene_coordinates::ScenePlanPoint::from_metres(
                        door.threshold_metres.metres()
                    )
                    .unwrap()
                )
                .unwrap()
                .elevation
                .metres()
                - plan.floor.elevation.metres())
            .abs()
                < 0.001
        );
    }
    let mut reversed = doors;
    reversed.reverse();
    let reordered = request(&source, &reversed).select().unwrap();
    assert_eq!(plan.surface.mesh(), reordered.surface.mesh());
}
#[test]
fn impossible_single_property_reports_the_same_member_and_exact_access_shortfall() {
    let source = source(0.7);
    let doors = doors();
    let error = request(&source, &doors).select().unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(7));
    assert_eq!(
        error.member_building_ids,
        [7].map(crate::scene_input::SceneBuildingId)
    );
    assert!(matches!(
        error.constraint,
        SupportConstraint::StairGoing | SupportConstraint::AccessGrade
    ));
    assert!(error.violation.actual_value() > error.violation.limit_value());
    assert!(error.violation.discrepancy_value() > 0.0);
    assert_eq!(error.boundary, SupportBoundary::StreetLanding);
    assert_eq!(
        *error.attempted_treatment,
        SupportGradingAttempt::SingleBuildingFloorAndEntrances
    );
}

#[test]
fn cut_banks_join_natural_ground_to_the_floor_without_adding_occupied_floor_height() {
    let source = source(0.1);
    let doors = doors();
    let plan = request(&source, &doors).select().unwrap();
    let foundation = plan
        .surface
        .foundations(&source, policy().embedment)
        .unwrap();
    assert!(!foundation.cut_faces.is_empty());
    let point = Vec3::new(3.0, 0.3, 6.0);
    assert!(foundation.cut_faces.iter().any(|triangle| {
        let edges = [triangle[1] - triangle[0], triangle[2] - triangle[0]];
        let normal = edges[0].cross(edges[1]).normalize();
        let plane_distance = (point - triangle[0]).dot(normal).abs();
        let edges_inside = (0..3).all(|i| {
            normal.dot((triangle[(i + 1) % 3] - triangle[i]).cross(point - triangle[i])) >= -0.001
        });
        plane_distance < 0.001 && edges_inside
    }));
    let terrain = BoundedSettlementTerrain::compile(
        std::slice::from_ref(&plan.surface),
        &source,
        policy().embedment,
    )
    .unwrap();
    let ray = Vec3::new(3.0, 0.3, 5.0);
    let hit = terrain
        .colliders()
        .unwrap()
        .iter()
        .filter_map(|collider| {
            collider
                .cast_ray(
                    Vec3::ZERO,
                    avian3d::prelude::Rotation::default(),
                    ray,
                    Vec3::Z,
                    2.0,
                    false,
                )
                .map(|(distance, _)| distance)
        })
        .min_by(f32::total_cmp)
        .unwrap();
    assert!((hit - 1.0).abs() < 0.001, "cut face ray distance {hit}");
    assert!(
        (terrain
            .highest_surface_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(Vec2::new(3.0, 5.0)).unwrap()
            )
            .unwrap()
            .elevation
            .metres()
            - plan.floor.elevation.metres())
        .abs()
            < 0.001
    );
    assert!(
        (terrain
            .highest_surface_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(Vec2::new(3.0, 6.001))
                    .unwrap()
            )
            .unwrap()
            .elevation
            .metres()
            - 0.6001)
            .abs()
            < 0.001
    );
}

#[test]
fn an_unsupported_owned_perimeter_is_rejected_instead_of_omitting_its_bank() {
    let source = source(0.1);
    let doors = doors();
    let mut plan = request(&source, &doors).select().unwrap();
    // Deliberately invalid negative fixture: the claimed cut extends a metre
    // beyond the actual floor. Generation must not accept its missing bank.
    let region = (plan.floor.contact)
        .resized(
            adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                Vec2::new(10.0, 12.0),
            )
            .unwrap(),
        )
        .unwrap();
    plan.surface.regions[0] = region;
    plan.surface.clipping_outlines[0] = region.corners().map(Vec2::as_dvec2).to_vec();
    let error = plan
        .surface
        .foundations(&source, policy().embedment)
        .unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(7));
    assert_eq!(
        error.member_building_ids,
        [7].map(crate::scene_input::SceneBuildingId)
    );
    assert_eq!(error.constraint, SupportConstraint::BoundaryCoverage);
    assert_eq!(error.boundary, SupportBoundary::PropertyReservation);
    assert!(error.violation.actual_value() > error.violation.limit_value());
    assert_eq!(
        error.violation.limit_value(),
        f64::from(policy().limits.contact_tolerance_metres())
    );
    assert!(error.violation.discrepancy_value() > 0.0);
    assert_eq!(
        *error.attempted_treatment,
        SupportGradingAttempt::SingleBuildingFloorAndEntrances
    );
}

#[test]
fn a_contained_centre_cannot_authorize_bearings_outside_its_property() {
    let source = source(0.1);
    let doors = doors();
    let mut request = request(&source, &doors);
    assert!(
        request
            .property
            .plot
            .contains(request.bearing.centre_metres())
    );
    let mut points: Vec<_> = request
        .bearing_region
        .outline()
        .vertices()
        .iter()
        .map(|p| p.metres())
        .collect();
    points[0].x = -6.02;
    request.bearing_region = FloorRegion::from_scene(
        crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
            points
                .into_iter()
                .map(|p| crate::scene_coordinates::ScenePlanPoint::from_metres(p).unwrap())
                .collect(),
        )
        .unwrap(),
    )
    .unwrap();
    let error = request.select().unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(7));
    assert_eq!(
        error.member_building_ids,
        [7].map(crate::scene_input::SceneBuildingId)
    );
    assert_eq!(error.constraint, SupportConstraint::Bearing);
    assert_eq!(error.boundary, SupportBoundary::PropertyReservation);
    assert!((error.violation.actual_value() - 0.02).abs() < 1e-6);
    assert!((error.violation.discrepancy_value() - 0.019).abs() < 1e-6);
    assert_eq!(error.location_metres.attempted_metres().x, -6.02);
}

#[test]
fn recessed_doorway_landings_share_the_floor_without_duplicate_bearings() {
    let source = source(0.1);
    let doors = doors();
    let plan = request(&source, &doors).select().unwrap();
    let terrain = BoundedSettlementTerrain::compile(
        std::slice::from_ref(&plan.surface),
        &source,
        policy().embedment,
    )
    .unwrap();
    let surface = GeographicSurface::from_triangles(terrain.support_triangles()).unwrap();
    let measured = surface
        .measure_region(
            &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                (plan.floor.contact.corners())
                    .iter()
                    .copied()
                    .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    // Both architectural thresholds sit 0.1 m inside the bearing. The apron
    // must join it, rather than contributing a second coplanar floor strip.
    assert!(
        (measured.covered_area_square_metres.square_metres()
            - measured.required_area_square_metres.square_metres())
        .abs()
            < 1e-6,
        "duplicate bearing area: {measured:?}"
    );
    for door in doors {
        assert!(
            (terrain
                .highest_surface_at(
                    crate::scene_coordinates::ScenePlanPoint::from_metres(
                        door.threshold_metres.metres()
                    )
                    .unwrap()
                )
                .unwrap()
                .elevation
                .metres()
                - plan.floor.elevation.metres())
            .abs()
                < 0.001
        );
        let approach = door
            .threshold_metres
            .translated(
                crate::scene_coordinates::PlanDisplacement::try_from(door.outward.vector() * 0.5)
                    .unwrap(),
            )
            .unwrap();
        assert!(
            (terrain
                .highest_surface_at(
                    crate::scene_coordinates::ScenePlanPoint::from_metres(approach.metres())
                        .unwrap()
                )
                .unwrap()
                .elevation
                .metres()
                - plan.floor.elevation.metres())
            .abs()
                < 0.001
        );
    }
    assert_eq!(
        terrain
            .highest_surface_at(
                crate::scene_coordinates::ScenePlanPoint::from_metres(Vec2::new(4.0, 11.0))
                    .unwrap()
            )
            .unwrap()
            .elevation
            .metres(),
        source
            .elevation_at(
                crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(4.0, 11.0)).unwrap()
            )
            .unwrap()
            .metres()
    );
}
