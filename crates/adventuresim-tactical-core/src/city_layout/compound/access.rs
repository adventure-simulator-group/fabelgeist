//! Immutable scene access segments admitted before route or support queries.
use super::*;
use crate::scene_coordinates::{PlanDisplacement, ScenePlanPoint};
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength};
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(opaque)]
#[serde(try_from = "AccessWire")]
pub struct CityAccessSegment {
    start_metres: ScenePlanPoint,
    end_metres: ScenePlanPoint,
    half_width_metres: PositiveLength,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessWire {
    start_metres: ScenePlanPoint,
    end_metres: ScenePlanPoint,
    half_width_metres: PositiveLength,
}
impl TryFrom<AccessWire> for CityAccessSegment {
    type Error = GeometryError;
    fn try_from(wire: AccessWire) -> Result<Self, GeometryError> {
        Self::new(wire.start_metres, wire.end_metres, wire.half_width_metres)
    }
}
impl CityAccessSegment {
    pub const JOIN_TOLERANCE_METRES: f32 = 0.02;
    pub fn new(
        start: ScenePlanPoint,
        end: ScenePlanPoint,
        half_width: PositiveLength,
    ) -> Result<Self, GeometryError> {
        if !(end.metres() - start.metres()).is_finite() {
            return Err(GeometryError::InvalidProjection);
        }
        Ok(Self {
            start_metres: start,
            end_metres: end,
            half_width_metres: half_width,
        })
    }
    pub fn start(self) -> ScenePlanPoint {
        self.start_metres
    }
    pub fn end(self) -> ScenePlanPoint {
        self.end_metres
    }
    pub fn half_width(self) -> PositiveLength {
        self.half_width_metres
    }
    /// Native scene east/north metre inputs for affine route clipping.
    pub fn start_metres(self) -> Vec2 {
        self.start_metres.metres()
    }
    pub fn end_metres(self) -> Vec2 {
        self.end_metres.metres()
    }
    pub fn half_width_metres(self) -> f32 {
        self.half_width_metres.metres()
    }
    pub fn update_endpoints(
        &mut self,
        start: ScenePlanPoint,
        end: ScenePlanPoint,
    ) -> Result<(), GeometryError> {
        *self = Self::new(start, end, self.half_width_metres)?;
        Ok(())
    }
    pub fn resize_half_width(&mut self, half_width: PositiveLength) -> Result<(), GeometryError> {
        *self = Self::new(self.start_metres, self.end_metres, half_width)?;
        Ok(())
    }
    pub fn with_endpoints(
        self,
        start: ScenePlanPoint,
        end: ScenePlanPoint,
    ) -> Result<Self, GeometryError> {
        Self::new(start, end, self.half_width_metres)
    }
    pub fn translated(
        self,
        start_delta: PlanDisplacement,
        end_delta: PlanDisplacement,
    ) -> Result<Self, GeometryError> {
        self.with_endpoints(
            self.start_metres.translated(start_delta)?,
            self.end_metres.translated(end_delta)?,
        )
    }
    pub fn ends_at(self, point: Vec2) -> bool {
        self.end_metres().distance(point) <= Self::JOIN_TOLERANCE_METRES
    }
    pub fn contains_centreline(self, point: Vec2) -> bool {
        let delta = self.end_metres() - self.start_metres();
        if delta.length_squared() <= f32::EPSILON {
            return false;
        }
        let fraction = (point - self.start_metres()).dot(delta) / delta.length_squared();
        (0.0..=1.0).contains(&fraction)
            && point.distance(self.start_metres() + delta * fraction) <= Self::JOIN_TOLERANCE_METRES
    }
}
/// Native packing translation is east/north metres; the first hook follows
/// the original street tangent while every other endpoint follows its owner.
pub(in crate::city_layout) fn translate_property_access(
    access: &mut [CityAccessSegment],
    delta: Vec2,
    tangent: Vec2,
) -> Result<(), GeometryError> {
    for (index, segment) in access.iter_mut().enumerate() {
        *segment = segment.translated(
            PlanDisplacement::try_from(if index == 0 {
                tangent * delta.dot(tangent)
            } else {
                delta
            })?,
            PlanDisplacement::try_from(delta)?,
        )?;
    }
    Ok(())
}
