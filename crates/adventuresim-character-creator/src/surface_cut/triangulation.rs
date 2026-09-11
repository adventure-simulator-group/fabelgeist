//! Conform Boolean fragments at shared edges before triangulating them.
use super::Polygon;

// The source positions and final correspondence weights are f32. Treating
// sub-precision repetitions as separate corners creates sliver triangles.
const BARYCENTRIC_COLLINEAR_EPSILON: f64 = 1e-6;

pub(super) fn conforming(polygons: Vec<Polygon>) -> Vec<[[f64; 3]; 3]> {
    let points = polygons.iter().flatten().copied().collect::<Vec<_>>();
    let mut result = Vec::new();
    for polygon in polygons {
        let mut boundary = Vec::new();
        for i in 0..polygon.len() {
            let (a, b) = (polygon[i], polygon[(i + 1) % polygon.len()]);
            let edge: [f64; 3] = std::array::from_fn(|axis| b[axis] - a[axis]);
            let squared: f64 = edge.iter().map(|v| v * v).sum();
            if squared < BARYCENTRIC_COLLINEAR_EPSILON.powi(2) {
                continue;
            }
            boundary.push(a);
            let mut splits = points
                .iter()
                .filter_map(|point| {
                    let t = (0..3)
                        .map(|axis| (point[axis] - a[axis]) * edge[axis])
                        .sum::<f64>()
                        / squared;
                    if !(BARYCENTRIC_COLLINEAR_EPSILON..1.0 - BARYCENTRIC_COLLINEAR_EPSILON)
                        .contains(&t)
                    {
                        return None;
                    }
                    (0..3)
                        .all(|axis| {
                            (point[axis] - a[axis] - t * edge[axis]).abs()
                                < BARYCENTRIC_COLLINEAR_EPSILON
                        })
                        .then_some((t, *point))
                })
                .collect::<Vec<_>>();
            splits.sort_by(|a, b| a.0.total_cmp(&b.0));
            splits.dedup_by(|a, b| (a.0 - b.0).abs() < BARYCENTRIC_COLLINEAR_EPSILON);
            boundary.extend(splits.into_iter().map(|(_, p)| p));
        }
        if boundary.len() == 3 {
            result.push([boundary[0], boundary[1], boundary[2]]);
        } else if boundary.len() > 3 {
            // A centre fan preserves every collinear boundary segment. A fan
            // from a corner can skip such segments as zero-area triangles.
            let centre = std::array::from_fn(|axis| {
                boundary.iter().map(|p| p[axis]).sum::<f64>() / boundary.len() as f64
            });
            for i in 0..boundary.len() {
                result.push([centre, boundary[i], boundary[(i + 1) % boundary.len()]]);
            }
        }
    }
    result
}
