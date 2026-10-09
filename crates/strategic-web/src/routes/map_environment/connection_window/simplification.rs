//! Iterative source polyline approximation at a physical display tolerance.
use adventuresim_world_schema::coordinates::terrain_projection::NativeTerrainOffset;

/// Native geometry kernel in continuous east/north metres. This is solely a
/// display approximation; source routing and its digest remain untouched.
pub(super) fn simplify(
    points: &[NativeTerrainOffset],
    tolerance_metres: f64,
) -> Vec<NativeTerrainOffset> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut pending = vec![[0, points.len() - 1]];
    while let Some([first, last]) = pending.pop() {
        let start = points[first];
        let direction = NativeTerrainOffset {
            east_metres: points[last].east_metres - start.east_metres,
            north_metres: points[last].north_metres - start.north_metres,
        };
        let length_squared = direction.east_metres.powi(2) + direction.north_metres.powi(2);
        let mut selected = None;
        let mut farthest_squared = tolerance_metres.powi(2);
        for (index, point) in points.iter().enumerate().take(last).skip(first + 1) {
            let east = point.east_metres - start.east_metres;
            let north = point.north_metres - start.north_metres;
            let parameter = if length_squared == 0.0 {
                0.0
            } else {
                ((east * direction.east_metres + north * direction.north_metres) / length_squared)
                    .clamp(0.0, 1.0)
            };
            let distance_squared = (east - parameter * direction.east_metres).powi(2)
                + (north - parameter * direction.north_metres).powi(2);
            if distance_squared > farthest_squared {
                farthest_squared = distance_squared;
                selected = Some(index);
            }
        }
        if let Some(index) = selected {
            keep[index] = true;
            pending.extend([[first, index], [index, last]]);
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(point, keep)| keep.then_some(*point))
        .collect()
}
