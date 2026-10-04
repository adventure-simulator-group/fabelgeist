use super::*;

pub(super) fn geographic_fixture() -> GeographicSurface {
    let value: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/goslar-1238.json"
    )))
    .unwrap();
    let triangles: Vec<[Vec3; 3]> =
        serde_json::from_value(value["geographic_triangles"].clone()).unwrap();
    GeographicSurface::from_triangles(triangles).unwrap()
}

pub(super) fn flat_source(plan: &CompoundSupportPlan, height: f32) -> Vec<[Vec3; 3]> {
    let mesh = plan.mesh();
    let minimum = mesh
        .positions
        .iter()
        .map(|p| p.xz())
        .fold(Vec2::splat(f32::INFINITY), Vec2::min)
        - Vec2::ONE;
    let maximum = mesh
        .positions
        .iter()
        .map(|p| p.xz())
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max)
        + Vec2::ONE;
    let corners = [
        minimum,
        Vec2::new(maximum.x, minimum.y),
        maximum,
        Vec2::new(minimum.x, maximum.y),
    ]
    .map(|p| Vec3::new(p.x, height, p.y));
    vec![
        [corners[0], corners[1], corners[2]],
        [corners[0], corners[2], corners[3]],
    ]
}

#[test]
fn goslar_foundations_are_closed_and_extend_below_complete_source_intersections() {
    let fixture = Fixture::load();
    let plan = fixture.plan(terraced());
    let foundations = plan
        .foundations(
            &geographic_fixture(),
            FoundationEmbedment::from_metres(0.2).unwrap(),
        )
        .unwrap();
    assert_eq!(foundations.property_id, CityPropertyId(1238));
    assert_eq!(foundations.member_building_ids, [1238, 17622]);
    assert!(foundations.volume_cubic_metres() > 1282.0);
    for (cell, triangles) in foundations
        .positions
        .as_chunks::<6>()
        .0
        .iter()
        .zip(foundations.solid_triangles.as_chunks::<8>().0)
    {
        for i in 0..3 {
            assert!(cell[i].y - cell[i + 3].y >= 0.1999);
        }
        let mut edges = std::collections::BTreeMap::new();
        for triangle in triangles {
            for i in 0..3 {
                let a = triangle[i];
                let b = triangle[(i + 1) % 3];
                let entry = edges.entry((a.min(b), a.max(b))).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        assert!(
            edges.values().all(|edge| *edge == (2, 0)),
            "open or misoriented foundation cell"
        );
    }
}

#[test]
fn a_coplanar_source_has_one_foundation_volume_instead_of_duplicate_zero_line_cells() {
    let mut fixture = Fixture::load();
    let height = fixture.levels.front.elevation.metres();
    fixture.levels.gate = elevation(height);
    fixture.levels.street = elevation(height);
    let plan = fixture.plan(CourtTreatment::Level);
    let source = GeographicSurface::from_triangles(flat_source(&plan, height)).unwrap();
    let foundation = plan
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap();
    let area: f64 = plan
        .mesh()
        .support_triangles
        .iter()
        .map(|indices| {
            let mesh = plan.mesh();
            let [a, b, c] = indices.map(|i| mesh.positions[i as usize].xz().as_dvec2());
            (b - a).perp_dot(c - a).abs() * 0.5
        })
        .sum();
    assert!((foundation.volume_cubic_metres() - area * 0.2).abs() < 0.01);
}

#[test]
fn geographic_iteration_and_winding_do_not_change_foundation_geometry() {
    let plan = Fixture::load().plan(terraced());
    let mut triangles = flat_source(&plan, 20.0);
    let first = plan
        .foundations(
            &GeographicSurface::from_triangles(triangles.clone()).unwrap(),
            FoundationEmbedment::from_metres(0.2).unwrap(),
        )
        .unwrap();
    triangles.reverse();
    for triangle in &mut triangles {
        triangle.swap(0, 1);
    }
    let second = plan
        .foundations(
            &GeographicSurface::from_triangles(triangles).unwrap(),
            FoundationEmbedment::from_metres(0.2).unwrap(),
        )
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn missing_geographic_coverage_reports_exact_property_and_square_metre_shortfall() {
    let plan = Fixture::load().plan(terraced());
    let source =
        GeographicSurface::from_triangles(flat_source(&plan, 20.0).into_iter().take(1)).unwrap();
    let error = plan
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(1238));
    assert_eq!(error.member_building_ids, [1238, 17622]);
    assert_eq!(error.constraint, SupportConstraint::SurfaceCoverage);
    assert_eq!(error.unit, SupportDiagnosticUnit::SquareMetres);
    assert!(error.shortfall > 0.1);
    assert_eq!(
        error.attempted_treatment,
        SupportGradingAttempt::Compound(terraced())
    );
}

#[test]
fn overlapping_geographic_triangles_are_rejected_instead_of_counted_as_support() {
    let plan = Fixture::load().plan(terraced());
    let mut triangles = flat_source(&plan, 20.0);
    triangles.extend(triangles.clone());
    let source = GeographicSurface::from_triangles(triangles).unwrap();
    let error = plan
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::SurfaceOverlap);
    assert_eq!(error.member_building_ids, [1238, 17622]);
    assert!(error.shortfall > 0.1);
}

#[test]
fn an_interior_source_peak_is_rejected_even_when_its_corners_fit_cut_fill_limits() {
    let plan = Fixture::load().plan(terraced());
    let square = flat_source(&plan, 20.0);
    let corners = [square[0][0], square[0][1], square[0][2], square[1][2]];
    let point = plan.reservation().centre_metres;
    let peak = Vec3::new(point.x, 28.0, point.y);
    let source =
        GeographicSurface::from_triangles((0..4).map(|i| [corners[i], corners[(i + 1) % 4], peak]))
            .unwrap();
    let error = plan
        .foundations(&source, FoundationEmbedment::from_metres(0.2).unwrap())
        .unwrap_err();
    assert_eq!(error.constraint, SupportConstraint::CutFill);
    assert_eq!(error.member_building_ids, [1238, 17622]);
    assert!(error.measured > error.permitted);
}

#[test]
fn bounded_replacement_removes_interior_ground_without_changing_surrounding_relief() {
    let plan = Fixture::load().doorway_plan();
    let source = geographic_fixture();
    let terrain = BoundedPropertyTerrain::compile(
        &plan,
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    for member in plan.member_support() {
        for x in 1..10 {
            for z in 1..10 {
                let local = (Vec2::new(x as f32, z as f32) / 10.0 - Vec2::splat(0.5))
                    * member.contact.dimensions_metres;
                let point =
                    member.contact.centre_metres + member.contact.orientation.local_to_world(local);
                let heights: Vec<_> = terrain.elevations_at(point).iter().collect();
                assert!(!heights.is_empty());
                assert!(
                    heights
                        .iter()
                        .all(|h| (h.metres() - member.elevation.metres()).abs() < 0.001),
                    "terrain intrudes into member {} at {point:?}: {heights:?}",
                    member.building_id
                );
            }
        }
    }
    for triangle in source.triangles() {
        let [a, b, c] = triangle;
        for u in 1..10 {
            for v in 1..10 - u {
                let point = a + (b - a) * (u as f32 / 10.0) + (c - a) * (v as f32 / 10.0);
                if plan.contains(point.xz()) {
                    continue;
                }
                let heights: Vec<_> = terrain.elevations_at(point.xz()).iter().collect();
                assert!(
                    !heights.is_empty(),
                    "outside terrain was removed at {point:?}"
                );
                assert!(
                    heights.iter().all(|h| (h.metres() - point.y).abs() < 0.001),
                    "outside terrain changed at {point:?}: {heights:?}"
                );
            }
        }
    }
}

#[test]
fn complete_triangles_reject_boundary_fill_that_a_containment_query_can_miss() {
    let mut fixture = Fixture::load();
    fixture.levels.front.elevation = elevation(21.192_965);
    fixture.levels.court = elevation(20.212_461);
    fixture.levels.rear.elevation = elevation(19.206_753);
    let plan = fixture.plan(CourtTreatment::Terraced(
        CourtStairLimits::new(0.19, 0.25, 1.0, 1.05, 0.5).unwrap(),
    ));
    let error = plan
        .foundations(
            &geographic_fixture(),
            FoundationEmbedment::from_metres(0.2).unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.property_id, CityPropertyId(1238));
    assert_eq!(error.member_building_ids, [1238, 17622]);
    assert_eq!(error.constraint, SupportConstraint::CutFill);
    assert_eq!(error.boundary, SupportBoundary::GeographicSurface);
    assert!(error.measured > 6.01);
    assert_eq!(error.permitted, 6.0);
    assert!(error.shortfall > 0.01);
}
