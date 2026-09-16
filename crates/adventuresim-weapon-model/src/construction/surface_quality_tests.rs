//! Sampling repairs preserve boundaries and cannot erase a physical crease.
use super::*;

fn close_interior() -> Region {
    Region {
        points: vec![
            [0.0, 0.0],
            [0.01, 0.0],
            [0.01, 0.01],
            [0.0, 0.01],
            [1e-8, 1e-8],
        ],
        triangles: vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        boundary: vec![0, 1, 2, 3],
    }
}

#[test]
fn interior_sampling_repair_preserves_exact_boundary_and_export_orientation() {
    let mut region = close_interior();
    let boundary = region.boundary.clone();
    let points = region.points.clone();
    region.improve_surface_cells(|_| (), |_| 0.001, 0.00002, 0.02);
    assert_eq!(region.boundary, boundary);
    assert_eq!(region.points, points);
    assert_eq!(region.triangles.len(), 2);
    let mut area = 0.0;
    for face in &region.triangles {
        let [a, b, c] = face.map(|i| region.points[i].map(|v| (v + 19.0) as f32 as f64));
        let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        assert!(cross > 0.0);
        area += cross / 2.0;
    }
    assert!((area - 0.0001).abs() < 1e-8);
}

#[test]
fn interior_sampling_repair_retains_crease_and_edge_budgets() {
    let mut crease = close_interior();
    let original = crease.triangles.clone();
    crease.improve_surface_cells(|p| p[0] > p[1], |_| 0.001, 0.00002, 0.02);
    assert_eq!(crease.triangles, original);
    let mut short_edges = close_interior();
    short_edges.improve_surface_cells(|_| (), |_| 0.001, 0.00002, 0.01);
    assert_eq!(short_edges.triangles, original);
}
