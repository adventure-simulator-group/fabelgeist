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
        limits: SupportLimits::new(0.65, 6.0, 0.001).unwrap(),
        stairs: CourtStairLimits::new(0.19, 0.25, 1.0, 1.05, 0.5).unwrap(),
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
            building_id: 7,
            plot: CityPlotBounds {
                centre_metres: Vec2::ZERO,
                dimensions_metres: Vec2::new(12.0, 24.0),
                orientation: BuildingOrientation::IDENTITY,
            },
        },
        bearing: CityPlotBounds {
            centre_metres: Vec2::ZERO,
            dimensions_metres: Vec2::new(8.0, 12.0),
            orientation: BuildingOrientation::IDENTITY,
        },
        bearing_outline: CityPlotBounds {
            centre_metres: Vec2::ZERO,
            dimensions_metres: Vec2::new(8.0, 12.0),
            orientation: BuildingOrientation::IDENTITY,
        }
        .corners()
        .to_vec(),
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
            threshold_metres: Vec2::new(0.0, -5.9),
            outward: -Vec2::Y,
        },
        DoorwaySupportBinding {
            entrance: BuildingEntranceId::Opening(OpeningAssemblyId(9)),
            support: adventuresim_building_generator::BuildingEntranceSupport::ArchitecturalFloor,
            threshold_metres: Vec2::new(0.0, 5.9),
            outward: Vec2::Y,
        },
    ]
}
#[test]
fn single_property_supports_all_doors_without_levelling_its_court_or_garden() {
    let source = source(0.1);
    let doors = doors();
    let plan = request(&source, &doors).select().unwrap();
    assert_eq!(plan.surface.member_building_ids(), [7]);
    let terrain = BoundedSettlementTerrain::compile(
        std::slice::from_ref(&plan.surface),
        &source,
        policy().embedment,
    )
    .unwrap();
    assert!(
        (terrain
            .highest_surface_at(Vec2::ZERO)
            .unwrap()
            .elevation
            .metres()
            - plan.floor.elevation.metres())
        .abs()
            < 0.001
    );
    let garden = Vec2::new(4.0, 11.0);
    assert!(!plan.surface.contains(garden));
    assert_eq!(
        terrain
            .highest_surface_at(garden)
            .unwrap()
            .elevation
            .metres(),
        source.elevation_at(garden).unwrap().metres()
    );
    for door in doors {
        assert!(
            (terrain
                .highest_surface_at(door.threshold_metres)
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
    assert_eq!(error.member_building_ids, [7]);
    assert!(matches!(
        error.constraint,
        SupportConstraint::StairGoing | SupportConstraint::AccessGrade
    ));
    assert!(error.measured > error.permitted);
    assert!(error.shortfall > 0.0);
    assert_eq!(error.boundary, SupportBoundary::StreetLanding);
    assert_eq!(
        error.attempted_treatment,
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
            .highest_surface_at(Vec2::new(3.0, 5.0))
            .unwrap()
            .elevation
            .metres()
            - plan.floor.elevation.metres())
        .abs()
            < 0.001
    );
    assert!(
        (terrain
            .highest_surface_at(Vec2::new(3.0, 6.001))
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
    let region = CityPlotBounds {
        dimensions_metres: Vec2::new(10.0, 12.0),
        ..plan.floor.contact
    };
    plan.surface.regions[0] = region;
    plan.surface.clipping_outlines[0] = region.corners().map(Vec2::as_dvec2).to_vec();
    let error = plan
        .surface
        .foundations(&source, policy().embedment)
        .unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(7));
    assert_eq!(error.member_building_ids, [7]);
    assert_eq!(error.constraint, SupportConstraint::BoundaryCoverage);
    assert_eq!(error.boundary, SupportBoundary::PropertyReservation);
    assert!(error.measured > error.permitted);
    assert_eq!(error.permitted, policy().limits.contact_tolerance_metres());
    assert!(error.shortfall > 0.0);
    assert_eq!(
        error.attempted_treatment,
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
            .contains(request.bearing.centre_metres)
    );
    request.bearing_outline[0].x = -6.02;
    let error = request.select().unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(7));
    assert_eq!(error.member_building_ids, [7]);
    assert_eq!(error.constraint, SupportConstraint::Bearing);
    assert_eq!(error.boundary, SupportBoundary::PropertyReservation);
    assert!((error.measured - 0.02).abs() < 1e-6);
    assert!((error.shortfall - 0.019).abs() < 1e-6);
    assert_eq!(error.location_metres.x, -6.02);
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
        .measure_region(&plan.floor.contact.corners())
        .unwrap();
    // Both architectural thresholds sit 0.1 m inside the bearing. The apron
    // must join it, rather than contributing a second coplanar floor strip.
    assert!(
        (measured.covered_area_square_metres - measured.required_area_square_metres).abs() < 1e-6,
        "duplicate bearing area: {measured:?}"
    );
    for door in doors {
        assert!(
            (terrain
                .highest_surface_at(door.threshold_metres)
                .unwrap()
                .elevation
                .metres()
                - plan.floor.elevation.metres())
            .abs()
                < 0.001
        );
        let approach = door.threshold_metres + door.outward * 0.5;
        assert!(
            (terrain
                .highest_surface_at(approach)
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
            .highest_surface_at(Vec2::new(4.0, 11.0))
            .unwrap()
            .elevation
            .metres(),
        source.elevation_at(Vec2::new(4.0, 11.0)).unwrap().metres()
    );
}
