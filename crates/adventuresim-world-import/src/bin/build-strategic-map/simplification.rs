//! Native longitude/latitude degree kernel for source water preprocessing.
use super::Point;

pub(super) const WATER_RING_TOLERANCE_DEGREES: f64 = 0.002;

struct FarthestPoint {
    distance_degrees: f64,
    index: usize,
}

/// Preserve the compiler's canonical water-ring simplification. Tolerance and
/// Euclidean distances are degrees at this source-format numerical boundary.
pub(super) fn simplify(points: &[Point], tolerance_degrees: f64) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut kept = vec![false; points.len()];
    kept[0] = true;
    kept[points.len() - 1] = true;
    let mut stack = vec![(0, points.len() - 1)];
    while let Some((start, end)) = stack.pop() {
        let mut farthest = FarthestPoint {
            distance_degrees: 0.0,
            index: start,
        };
        for index in start + 1..end {
            let distance = segment_distance(&points[index], &points[start], &points[end]);
            if distance > farthest.distance_degrees {
                farthest = FarthestPoint {
                    distance_degrees: distance,
                    index,
                };
            }
        }
        if farthest.distance_degrees > tolerance_degrees {
            kept[farthest.index] = true;
            stack.push((start, farthest.index));
            stack.push((farthest.index, end));
        }
    }
    points
        .iter()
        .zip(kept)
        .filter_map(|(point, keep)| keep.then_some(point.clone()))
        .collect()
}

fn segment_distance(point: &Point, start: &Point, end: &Point) -> f64 {
    let [longitude, latitude] = point.0;
    let [start_longitude, start_latitude] = start.0;
    let [end_longitude, end_latitude] = end.0;
    let length =
        (end_longitude - start_longitude).powi(2) + (end_latitude - start_latitude).powi(2);
    if length == 0.0 {
        return ((longitude - start_longitude).powi(2) + (latitude - start_latitude).powi(2))
            .sqrt();
    }
    let position = (((longitude - start_longitude) * (end_longitude - start_longitude)
        + (latitude - start_latitude) * (end_latitude - start_latitude))
        / length)
        .clamp(0.0, 1.0);
    ((longitude - (start_longitude + position * (end_longitude - start_longitude))).powi(2)
        + (latitude - (start_latitude + position * (end_latitude - start_latitude))).powi(2))
    .sqrt()
}
