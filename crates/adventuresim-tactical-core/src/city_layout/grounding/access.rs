//! Continuous approach intervals respect the property and near street half.
use super::*;
use crate::city_layout::CityStreetPatch;
use crate::scene_input::BuildingOrientation;
pub(in crate::city_layout) fn access_regions(
    plot: CityPlotBounds,
    streets: &[CityStreetPatch],
    edge: Vec2,
) -> Vec<Vec<bevy::math::DVec2>> {
    let mut regions = vec![plot];
    for street in streets {
        let CityStreetPatch::Corridor {
            start_metres,
            end_metres,
            half_width_metres,
            ..
        } = *street
        else {
            continue;
        };
        let tangent = (end_metres - start_metres).normalize();
        let normal = Vec2::new(-tangent.y, tangent.x);
        let side = (edge - start_metres).dot(normal).signum();
        if side == 0.0 {
            continue;
        }
        regions.push(CityPlotBounds {
            centre_metres: (start_metres + end_metres) * 0.5
                + normal * side * half_width_metres * 0.5,
            dimensions_metres: Vec2::new(start_metres.distance(end_metres), half_width_metres),
            orientation: BuildingOrientation::from_frontage_tangent(tangent)
                .expect("finite nonzero bound doorway/street axis"),
        });
    }
    regions
        .into_iter()
        .map(|r| r.corners().map(Vec2::as_dvec2).to_vec())
        .collect()
}

pub(in crate::city_layout) fn available_run(
    edge: Vec2,
    direction: Vec2,
    offset: Vec2,
    regions: &[Vec<bevy::math::DVec2>],
    maximum: f32,
    tolerance: f32,
) -> f32 {
    // A clear strip may straddle a junction between street segments. Subtract
    // their union, rather than requiring both strip edges inside one segment.
    let maximum = if maximum.is_finite() {
        maximum
    } else {
        regions
            .iter()
            .flat_map(|r| r.iter().copied())
            .map(|p| (p - edge.as_dvec2()).dot(direction.as_dvec2()) as f32)
            .fold(0.0_f32, f32::max)
    };
    if maximum <= 0.0 {
        return 0.0;
    }
    let points = [
        edge - offset,
        edge + direction * maximum - offset,
        edge + direction * maximum + offset,
        edge + offset,
    ];
    let mut remaining = vec![points.map(Vec2::as_dvec2).to_vec()];
    for region in regions {
        remaining = remaining
            .into_iter()
            .flat_map(|polygon| crate::city_layout::grounding::planar::subtract(polygon, region))
            .collect();
    }
    remaining
        .into_iter()
        .filter(|polygon| minimum_width(polygon) > f64::from(tolerance))
        .flat_map(|polygon| polygon.into_iter())
        .map(|p| (p - edge.as_dvec2()).dot(direction.as_dvec2()) as f32)
        .fold(maximum, f32::min)
        .max(0.0)
}

fn minimum_width(polygon: &[bevy::math::DVec2]) -> f64 {
    if super::planar::signed_area(polygon).abs() <= f64::EPSILON {
        return 0.0;
    }
    (0..polygon.len())
        .filter_map(|i| {
            let edge = polygon[(i + 1) % polygon.len()] - polygon[i];
            (edge.length_squared() > f64::EPSILON).then(|| {
                let normal = edge.perp().normalize();
                let lo = polygon
                    .iter()
                    .map(|p| p.dot(normal))
                    .fold(f64::INFINITY, f64::min);
                let hi = polygon
                    .iter()
                    .map(|p| p.dot(normal))
                    .fold(f64::NEG_INFINITY, f64::max);
                hi - lo
            })
        })
        .fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
#[path = "access/tests.rs"]
mod tests;
