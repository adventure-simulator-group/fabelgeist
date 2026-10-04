use super::*;
use crate::city_layout::grounding::tests::Fixture;

fn independent_margin_queries(
    index: &SupportQueryIndex,
    surface: &BoundedSettlementTerrain,
    point: Vec2,
    tolerance: f32,
) -> Vec<[Vec3; 3]> {
    let rounding = GroundTriangle::represented_point_tolerance(point);
    let mut candidates = Vec::new();
    let mut owners = Vec::new();
    index.visit_candidates(0, point, tolerance, &mut |reference| {
        let triangle = GroundTriangle::new(reference.points(surface)).unwrap();
        match reference {
            SupportTriangleRef::Natural(_) => {
                if triangle.contains(point, rounding) && triangle.contains(point, tolerance) {
                    candidates.push((reference, triangle));
                }
            }
            SupportTriangleRef::Foundation { owner, .. } => {
                if triangle.contains(point, rounding) && !owners.contains(&owner) {
                    owners.push(owner);
                }
                if triangle.contains(point, tolerance) {
                    candidates.push((reference, triangle));
                }
            }
        }
    });
    candidates
        .into_iter()
        .filter_map(|(reference, triangle)| {
            let include = match reference {
                SupportTriangleRef::Natural(_) => true,
                SupportTriangleRef::Foundation { owner, .. } => owners.contains(&owner),
            };
            include.then(|| triangle.points())
        })
        .collect()
}

#[test]
fn nested_margin_reuse_preserves_all_candidates_and_their_shared_edge_order() {
    let fixture = Fixture::load_965();
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let surface = BoundedSettlementTerrain::compile(
        &[plan.support_surface()],
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    let mut points = Vec::new();
    for triangle in surface.support_triangles() {
        for point in triangle.into_iter().chain([
            (triangle[0] + triangle[1]) * 0.5,
            (triangle[0] + triangle[1] + triangle[2]) / 3.0,
        ]) {
            points.extend([
                point.xz(),
                point.xz() + Vec2::splat(0.0005),
                point.xz() - Vec2::splat(0.0005),
            ]);
        }
    }
    for tolerance in [0.001, 0.000001, 0.00000001] {
        for &point in &points {
            let expected = independent_margin_queries(&surface.query, &surface, point, tolerance);
            let actual: Vec<_> = surface
                .query
                .triangles_at(&surface, point, tolerance)
                .map(|t| t.points())
                .collect();
            assert_eq!(
                actual,
                expected,
                "point {point:?}, contact margin {tolerance}, rounding {}",
                GroundTriangle::represented_point_tolerance(point)
            );
        }
    }
}
