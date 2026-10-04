//! Separating axes reuse the complete measured rectangles and convex bearings.
use super::*;

#[derive(Clone, Copy)]
pub(super) enum Envelope<'a> {
    Rectangle(CityPlotBounds),
    Polygon(&'a [Vec2]),
    Ground(&'a ParcelGeometry),
}

impl Envelope<'_> {
    pub(super) fn axes(self) -> Vec<DVec2> {
        match self {
            Self::Rectangle(bounds) => [Vec2::X, Vec2::Y]
                .map(|axis| bounds.orientation.local_to_world(axis).as_dvec2())
                .to_vec(),
            Self::Ground(geometry) => Self::Rectangle(geometry.reservation).axes(),
            Self::Polygon(points) => points
                .iter()
                .zip(points.iter().cycle().skip(1))
                .take(points.len())
                .map(|(a, b)| (b.as_dvec2() - a.as_dvec2()).perp().normalize())
                .collect(),
        }
    }
    pub(super) fn projection(self, axis: DVec2) -> (f64, f64) {
        match self {
            Self::Rectangle(bounds) => {
                let [x, y] =
                    [Vec2::X, Vec2::Y].map(|a| bounds.orientation.local_to_world(a).as_dvec2());
                let half = bounds.dimensions_metres.as_dvec2() * 0.5;
                let radius = half.x * axis.dot(x).abs() + half.y * axis.dot(y).abs();
                let centre = axis.dot(bounds.centre_metres.as_dvec2());
                (centre - radius, centre + radius)
            }
            Self::Polygon(points) => points
                .iter()
                .map(|point| axis.dot(point.as_dvec2()))
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                    (low.min(value), high.max(value))
                }),
            Self::Ground(geometry) => {
                let (low, high) = Self::Rectangle(geometry.reservation).projection(axis);
                geometry
                    .bearings
                    .iter()
                    .flatten()
                    .map(|point| axis.dot(point.as_dvec2()))
                    .fold((low, high), |(low, high), value| {
                        (low.min(value), high.max(value))
                    })
            }
        }
    }
}
