use super::*;

#[test]
fn exact_surface_edge_limit_refines_conservatively_across_rounding_and_translation() {
    let a = [0.02958039891549808, 0.095];
    let b = [0.017748239349298846, 0.09300000000000001];
    for translation in [0.0, 1.0, 19.0] {
        for shift in [-f64::EPSILON, 0.0, f64::EPSILON] {
            let translated_a = [a[0] + translation + shift, a[1] + translation];
            let translated_b = b.map(|v| v + translation);
            assert!(!surface_refinement::within_surface_edge(
                translated_a,
                translated_b,
                0.012
            ));
            let midpoint =
                std::array::from_fn(|axis| (translated_a[axis] + translated_b[axis]) / 2.0);
            assert!(surface_refinement::within_surface_edge(
                translated_a,
                midpoint,
                0.012
            ));
        }
    }
}

fn blank_with_small_scroll() -> Vec<PlanarPoint> {
    let mut outline = vec![[0.0, 0.0], [0.1, 0.0], [0.1, 1.0], [0.0, 1.0]];
    outline.extend([[0.0, 0.31], [-0.044, 0.31], [-0.046, 0.592]]);
    for (radius, reverse) in [(0.0035, false), (0.002, true)] {
        for index in 0..=24 {
            let progress = if reverse { 24 - index } else { index } as f64 / 24.0;
            let angle = (-30.0 + 320.0 * progress).to_radians();
            outline.push([-0.05 + radius * angle.cos(), 0.6 + radius * angle.sin()]);
        }
    }
    outline.extend([[-0.048, 0.592], [-0.05, 0.30], [0.0, 0.30]]);
    outline
}

#[test]
fn small_scroll_refines_within_budget_without_crossing_surface_partitions() {
    let outline = blank_with_small_scroll();
    let height = |[x, y]: PlanarPoint| 0.003 + 0.003 * x * y;
    let cell = |p: PlanarPoint| p[1] >= 0.5;
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let mut region = Region::triangulate(&outline, false).unwrap();
        region.partition(PlanarCut::Axial(0.5)).unwrap();
        region.remesh_cells(cell).unwrap();
        let maximum_edge = detail.error(0.012);
        let maximum_deviation = detail.error(0.00005);
        region
            .refine_rational_surface(cell, height, maximum_deviation, maximum_edge)
            .unwrap();
        construction_budget((2 * region.triangles.len() + 2 * region.boundary.len()) as f64)
            .unwrap();
        for &face in &region.triangles {
            let points = face.map(|i| region.points[i]);
            assert!(points.iter().all(|p| p[1] <= 0.5) || points.iter().all(|p| p[1] >= 0.5));
            assert!(
                surface_refinement::surface_deviation(points, &height) <= maximum_deviation / 2.0
            );
            for i in 0..3 {
                let [a, b] = [points[i], points[(i + 1) % 3]];
                assert!((a[0] - b[0]).hypot(a[1] - b[1]) <= maximum_edge);
            }
        }
        for point in &outline {
            assert!(region.boundary.iter().any(|&i| region.points[i] == *point));
        }
    }
}
