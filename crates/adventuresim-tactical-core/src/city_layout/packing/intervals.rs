//! Complete translation intervals avoid grid-search gaps between properties.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct FrontageInterval {
    pub minimum_metres: f64,
    pub maximum_metres: f64,
}

#[derive(Clone, Copy)]
pub(super) enum PackingClearance {
    PropertyBoundary,
    BuildingBody,
    GardenWorking,
    GardenPlant,
}
impl PackingClearance {
    pub(in crate::city_layout::packing) fn metres(self) -> f64 {
        match self {
            Self::PropertyBoundary => CityPlotBounds::COORDINATE_TOLERANCE_METRES,
            Self::GardenWorking => 0.0,
            Self::GardenPlant => f64::from(gardens::GARDEN_LEAF_WIND_CLEARANCE_METRES),
            Self::BuildingBody => {
                f64::from(PARTY_WALL_CLEARANCE_METRES) + CityPlotBounds::COORDINATE_TOLERANCE_METRES
            }
        }
    }
}

impl FrontageInterval {
    pub(super) fn with_half_plane(self, origin: f64, rate: f64) -> Option<Self> {
        let mut interval = self;
        if rate > f64::EPSILON {
            interval.minimum_metres = interval.minimum_metres.max(-origin / rate);
        } else if rate < -f64::EPSILON {
            interval.maximum_metres = interval.maximum_metres.min(-origin / rate);
        } else if origin < 0.0 {
            return None;
        }
        (interval.minimum_metres <= interval.maximum_metres).then_some(interval)
    }

    pub(super) fn intersection(self, other: Self) -> Option<Self> {
        let result = Self {
            minimum_metres: self.minimum_metres.max(other.minimum_metres),
            maximum_metres: self.maximum_metres.min(other.maximum_metres),
        };
        (result.minimum_metres <= result.maximum_metres).then_some(result)
    }

    pub(super) fn intersects(self, other: Self) -> bool {
        self.minimum_metres < other.maximum_metres && self.maximum_metres > other.minimum_metres
    }

    pub(super) fn without(self, other: Self) -> Vec<Self> {
        if !self.intersects(other) {
            return vec![self];
        }
        let mut free = Vec::new();
        if self.minimum_metres < other.minimum_metres {
            free.push(Self {
                minimum_metres: self.minimum_metres,
                maximum_metres: other.minimum_metres.min(self.maximum_metres),
            });
        }
        if self.maximum_metres > other.maximum_metres {
            free.push(Self {
                minimum_metres: other.maximum_metres.max(self.minimum_metres),
                maximum_metres: self.maximum_metres,
            });
        }
        free
    }

    pub(super) fn nearest_origin(&self) -> f64 {
        0.0_f64.clamp(self.minimum_metres, self.maximum_metres)
    }

    pub(super) fn overlap_polygons(
        first: &[Vec2],
        second: &[Vec2],
        tangent: Vec2,
        clearance: PackingClearance,
        second_translation_metres: DVec2,
    ) -> Option<Self> {
        let axes = [first, second]
            .into_iter()
            .flat_map(|outline| {
                outline
                    .iter()
                    .zip(outline.iter().cycle().skip(1))
                    .take(outline.len())
            })
            .map(|(a, b)| (b.as_dvec2() - a.as_dvec2()).perp().normalize());
        let interval = |outline: &[Vec2], axis: DVec2| {
            outline
                .iter()
                .map(|point| point.as_dvec2().dot(axis))
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), p| {
                    (min.min(p), max.max(p))
                })
        };
        let mut forbidden = Self {
            minimum_metres: f64::NEG_INFINITY,
            maximum_metres: f64::INFINITY,
        };
        for axis in axes {
            let (a, b) = interval(first, axis);
            let (c, d) = interval(second, axis);
            let offset = second_translation_metres.dot(axis);
            let (c, d) = (c + offset, d + offset);
            let rate = tangent.as_dvec2().dot(axis);
            forbidden = forbidden.with_half_plane(b + clearance.metres() - c, rate)?;
            forbidden = forbidden.with_half_plane(d + clearance.metres() - a, -rate)?;
        }
        Some(forbidden)
    }

    pub(super) fn overlap_displacements(
        first: CityPlotBounds,
        second: CityPlotBounds,
        tangent: Vec2,
        clearance: PackingClearance,
        second_translation_metres: DVec2,
    ) -> Option<Self> {
        let axes = |p: CityPlotBounds| {
            [
                p.orientation.local_to_world(Vec2::X).as_dvec2(),
                p.orientation.local_to_world(Vec2::Y).as_dvec2(),
            ]
        };
        let a = axes(first);
        let b = axes(second);
        let radius = |p: CityPlotBounds, axes: [DVec2; 2], axis: DVec2| {
            f64::from(p.dimensions_metres.x) * 0.5 * axis.dot(axes[0]).abs()
                + f64::from(p.dimensions_metres.y) * 0.5 * axis.dot(axes[1]).abs()
        };
        let delta = second.centre_metres.as_dvec2() - first.centre_metres.as_dvec2()
            + second_translation_metres;
        let mut forbidden = Self {
            minimum_metres: f64::NEG_INFINITY,
            maximum_metres: f64::INFINITY,
        };
        for axis in a.into_iter().chain(b) {
            let separation = delta.dot(axis);
            let reach = radius(first, a, axis) + radius(second, b, axis) + clearance.metres();
            let rate = -tangent.as_dvec2().dot(axis);
            forbidden = forbidden.with_half_plane(separation + reach, rate)?;
            forbidden = forbidden.with_half_plane(reach - separation, -rate)?;
        }
        Some(forbidden)
    }
}
