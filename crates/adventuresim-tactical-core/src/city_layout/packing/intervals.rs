//! Complete translation intervals avoid grid-search gaps between properties.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "IntervalWire")]
pub struct FrontageInterval {
    minimum_metres: f64,
    maximum_metres: f64,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct IntervalWire {
    minimum_metres: f64,
    maximum_metres: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error, serde::Serialize)]
pub enum FrontageIntervalError {
    #[error("frontage bounds contain NaN")]
    NotANumber,
    #[error("frontage bounds are reversed")]
    ReversedBounds,
    #[error("frontage constraint arithmetic is invalid")]
    InvalidArithmetic,
}

#[derive(Clone, Copy)]
pub(super) enum PackingClearance {
    PropertyBoundary,
    BuildingBody,
    GardenWorking,
    GardenPlant,
}

impl TryFrom<IntervalWire> for FrontageInterval {
    type Error = FrontageIntervalError;
    fn try_from(wire: IntervalWire) -> Result<Self, Self::Error> {
        Self::new(wire.minimum_metres, wire.maximum_metres)
    }
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
    /// Infinite bounds are purposeful intermediate solver limits; selected
    /// translations are admitted separately as finite FrontageDisplacement.
    pub fn new(minimum_metres: f64, maximum_metres: f64) -> Result<Self, FrontageIntervalError> {
        if minimum_metres.is_nan() || maximum_metres.is_nan() {
            return Err(FrontageIntervalError::NotANumber);
        }
        if minimum_metres > maximum_metres {
            return Err(FrontageIntervalError::ReversedBounds);
        }
        Ok(Self {
            minimum_metres,
            maximum_metres,
        })
    }
    pub const fn unbounded() -> Self {
        Self {
            minimum_metres: f64::NEG_INFINITY,
            maximum_metres: f64::INFINITY,
        }
    }
    pub fn minimum_metres(self) -> f64 {
        self.minimum_metres
    }
    pub fn maximum_metres(self) -> f64 {
        self.maximum_metres
    }

    pub(super) fn with_half_plane(
        self,
        origin: f64,
        rate: f64,
    ) -> Result<Option<Self>, FrontageIntervalError> {
        if origin.is_nan() || !rate.is_finite() {
            return Err(FrontageIntervalError::InvalidArithmetic);
        }
        let mut interval = self;
        if rate > f64::EPSILON {
            interval.minimum_metres = interval.minimum_metres.max(-origin / rate);
        } else if rate < -f64::EPSILON {
            interval.maximum_metres = interval.maximum_metres.min(-origin / rate);
        } else if origin < 0.0 {
            return Ok(None);
        }
        Ok((interval.minimum_metres <= interval.maximum_metres).then_some(interval))
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
        first: &[ScenePlanPoint],
        second: &[ScenePlanPoint],
        tangent: Vec2,
        clearance: PackingClearance,
        second_translation_metres: DVec2,
    ) -> Result<Option<Self>, FrontageIntervalError> {
        let axes = [first, second]
            .into_iter()
            .flat_map(|outline| {
                outline
                    .iter()
                    .zip(outline.iter().cycle().skip(1))
                    .take(outline.len())
            })
            .map(|(a, b)| {
                (b.metres().as_dvec2() - a.metres().as_dvec2())
                    .perp()
                    .normalize()
            });
        let interval = |outline: &[ScenePlanPoint], axis: DVec2| {
            outline
                .iter()
                .map(|point| point.metres().as_dvec2().dot(axis))
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), p| {
                    (min.min(p), max.max(p))
                })
        };
        let mut forbidden = Self::unbounded();
        for axis in axes {
            let (a, b) = interval(first, axis);
            let (c, d) = interval(second, axis);
            let offset = second_translation_metres.dot(axis);
            let (c, d) = (c + offset, d + offset);
            let rate = tangent.as_dvec2().dot(axis);
            let Some(narrowed) = forbidden.with_half_plane(b + clearance.metres() - c, rate)?
            else {
                return Ok(None);
            };
            forbidden = narrowed;
            let Some(narrowed) = forbidden.with_half_plane(d + clearance.metres() - a, -rate)?
            else {
                return Ok(None);
            };
            forbidden = narrowed;
        }
        Ok(Some(forbidden))
    }

    pub(super) fn overlap_displacements(
        first: CityPlotBounds,
        second: CityPlotBounds,
        tangent: Vec2,
        clearance: PackingClearance,
        second_translation_metres: DVec2,
    ) -> Result<Option<Self>, FrontageIntervalError> {
        let axes = |p: CityPlotBounds| {
            [
                p.orientation().local_to_world(Vec2::X).as_dvec2(),
                p.orientation().local_to_world(Vec2::Y).as_dvec2(),
            ]
        };
        let a = axes(first);
        let b = axes(second);
        let radius = |p: CityPlotBounds, axes: [DVec2; 2], axis: DVec2| {
            f64::from(p.dimensions_metres().x) * 0.5 * axis.dot(axes[0]).abs()
                + f64::from(p.dimensions_metres().y) * 0.5 * axis.dot(axes[1]).abs()
        };
        let delta = second.centre_metres().as_dvec2() - first.centre_metres().as_dvec2()
            + second_translation_metres;
        let mut forbidden = Self::unbounded();
        for axis in a.into_iter().chain(b) {
            let separation = delta.dot(axis);
            let reach = radius(first, a, axis) + radius(second, b, axis) + clearance.metres();
            let rate = -tangent.as_dvec2().dot(axis);
            let Some(narrowed) = forbidden.with_half_plane(separation + reach, rate)? else {
                return Ok(None);
            };
            forbidden = narrowed;
            let Some(narrowed) = forbidden.with_half_plane(reach - separation, -rate)? else {
                return Ok(None);
            };
            forbidden = narrowed;
        }
        Ok(Some(forbidden))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_admit_infinity_and_contact_but_reject_nan_and_reversal() {
        let unbounded = FrontageInterval::new(f64::NEG_INFINITY, f64::INFINITY).unwrap();
        assert_eq!(unbounded, FrontageInterval::unbounded());
        assert_eq!(
            unbounded
                .with_half_plane(2.0, 1.0)
                .unwrap()
                .unwrap()
                .minimum_metres(),
            -2.0
        );
        assert!(matches!(
            FrontageInterval::new(f64::NAN, 1.0),
            Err(FrontageIntervalError::NotANumber)
        ));
        assert!(matches!(
            FrontageInterval::new(1.0, 0.0),
            Err(FrontageIntervalError::ReversedBounds)
        ));
        let contact = FrontageInterval::new(0.0, 0.0).unwrap();
        assert!(contact.intersection(contact).is_some());
        assert!(!contact.intersects(contact));
        assert_eq!(contact.with_half_plane(-1.0, 0.0), Ok(None));
        assert_eq!(
            contact.with_half_plane(f64::NAN, 1.0),
            Err(FrontageIntervalError::InvalidArithmetic)
        );
    }
    #[test]
    fn decoding_validates_the_same_bounds() {
        assert!(
            serde_json::from_str::<FrontageInterval>(
                r#"{"minimum_metres":2.0,"maximum_metres":1.0}"#
            )
            .is_err()
        );
        let infinite = FrontageInterval::unbounded();
        assert_eq!(
            postcard::from_bytes::<FrontageInterval>(&postcard::to_allocvec(&infinite).unwrap())
                .unwrap(),
            infinite
        );
        let invalid = postcard::to_allocvec(&(f64::NAN, 1.0_f64)).unwrap();
        assert!(postcard::from_bytes::<FrontageInterval>(&invalid).is_err());
        let interval = FrontageInterval::new(-2.0, 3.0).unwrap();
        assert_eq!(
            serde_json::from_str::<FrontageInterval>(&serde_json::to_string(&interval).unwrap())
                .unwrap(),
            interval
        );
    }
}
