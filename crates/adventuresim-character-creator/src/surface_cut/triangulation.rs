//! Conform Boolean fragments at shared edges before triangulating them.
use super::Polygon;

// The source positions and final correspondence weights are f32. Treating
// sub-precision repetitions as separate corners creates sliver triangles.
pub(super) const BARYCENTRIC_COLLINEAR_EPSILON: f64 = 1e-6;

pub(super) struct ConformingTriangle {
    pub polygon: usize,
    pub corners: [[f64; 3]; 3],
}

pub(super) fn conforming(polygons: Vec<Polygon>, shared: &[[f64; 3]]) -> Vec<ConformingTriangle> {
    let points = polygons
        .iter()
        .flatten()
        .copied()
        .chain(shared.iter().copied())
        .collect::<Vec<_>>();
    let mut result = Vec::new();
    for (polygon_index, polygon) in polygons.into_iter().enumerate() {
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
        triangulate(boundary, polygon_index, &mut result);
    }
    result
}

/// Convex clipping fragments may contain collinear sewing samples. Ear removal
/// keeps every boundary segment while avoiding two extra faces and a new centre
/// for each ordinary quad. No diagonal may pass through another boundary point.
fn triangulate(mut boundary: Polygon, polygon: usize, result: &mut Vec<ConformingTriangle>) {
    let orientation = boundary
        .iter()
        .zip(boundary.iter().cycle().skip(1))
        .map(|(a, b)| a[0] * b[1] - a[1] * b[0])
        .sum::<f64>()
        .signum();
    while boundary.len() > 3 {
        let ear = (0..boundary.len())
            .filter_map(|index| {
                let previous = (index + boundary.len() - 1) % boundary.len();
                let next = (index + 1) % boundary.len();
                let corners = [boundary[previous], boundary[index], boundary[next]];
                let area = orientation * turn(corners[0], corners[1], corners[2]);
                if area <= BARYCENTRIC_COLLINEAR_EPSILON.powi(2) {
                    return None;
                }
                let occupied = boundary.iter().enumerate().any(|(other, point)| {
                    if [previous, index, next].contains(&other) {
                        return false;
                    }
                    (0..3).all(|edge| {
                        orientation * turn(corners[edge], corners[(edge + 1) % 3], *point)
                            >= -BARYCENTRIC_COLLINEAR_EPSILON.powi(2)
                    })
                });
                (!occupied).then_some((index, area, corners))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));
        let Some((index, _, corners)) = ear else {
            // Nearly coincident source weights can make all diagonals ambiguous.
            // A convex centre fan still retains the complete sewing boundary.
            let centre = std::array::from_fn(|axis| {
                boundary.iter().map(|p| p[axis]).sum::<f64>() / boundary.len() as f64
            });
            for index in 0..boundary.len() {
                result.push(ConformingTriangle {
                    polygon,
                    corners: [
                        centre,
                        boundary[index],
                        boundary[(index + 1) % boundary.len()],
                    ],
                });
            }
            return;
        };
        result.push(ConformingTriangle { polygon, corners });
        boundary.remove(index);
    }
    if let [a, b, c] = boundary.as_slice() {
        result.push(ConformingTriangle {
            polygon,
            corners: [*a, *b, *c],
        });
    }
}

/// Signed doubled area in the native source-triangle barycentric chart.
fn turn(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
