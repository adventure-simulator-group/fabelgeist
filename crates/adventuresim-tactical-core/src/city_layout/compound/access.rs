//! Immutable scene access segments admitted before route or support queries.
use super::*;
use crate::scene_coordinates::{PlanDisplacement, ScenePlanPoint};
use adventuresim_building_generator::spatial_geometry::{
    GeometryError, GeometryResult, PositiveLength,
};
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
    fn try_from(wire: AccessWire) -> Result<Self, Self::Error> {
        Self::new(wire.start_metres, wire.end_metres, wire.half_width_metres)
    }
}
impl CityAccessSegment {
    pub const JOIN_TOLERANCE_METRES: f32 = 0.02;
    pub fn new(
        start: ScenePlanPoint,
        end: ScenePlanPoint,
        half_width: PositiveLength,
    ) -> GeometryResult<Self> {
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
    ) -> GeometryResult<()> {
        *self = Self::new(start, end, self.half_width_metres)?;
        Ok(())
    }
    pub fn resize_half_width(&mut self, half_width: PositiveLength) -> GeometryResult<()> {
        *self = Self::new(self.start_metres, self.end_metres, half_width)?;
        Ok(())
    }
    pub fn with_endpoints(
        self,
        start: ScenePlanPoint,
        end: ScenePlanPoint,
    ) -> GeometryResult<Self> {
        Self::new(start, end, self.half_width_metres)
    }
    pub fn translated(
        self,
        start_delta: PlanDisplacement,
        end_delta: PlanDisplacement,
    ) -> GeometryResult<Self> {
        self.with_endpoints(
            self.start_metres.translated(start_delta)?,
            self.end_metres.translated(end_delta)?,
        )
    }
    /// Threshold binding keeps its scene frame through the domain predicate.
    /// ```compile_fail
    /// use adventuresim_tactical_core::city_layout::CityAccessSegment;
    /// use bevy::math::Vec2;
    /// fn unadmitted(route: CityAccessSegment, point: Vec2) -> bool {
    ///     route.ends_at(point)
    /// }
    /// ```
    pub fn ends_at(self, point: ScenePlanPoint) -> bool {
        self.end_metres().distance(point.metres()) <= Self::JOIN_TOLERANCE_METRES
    }
    /// Gate binding consumes an admitted scene point before affine clipping.
    /// ```compile_fail
    /// use adventuresim_tactical_core::city_layout::CityAccessSegment;
    /// use adventuresim_building_generator::plan_geometry::ArchitecturalPlanPoint;
    /// fn wrong_frame(route: CityAccessSegment, point: ArchitecturalPlanPoint) -> bool {
    ///     route.contains_centreline(point)
    /// }
    /// ```
    pub fn contains_centreline(self, point: ScenePlanPoint) -> bool {
        let point = point.metres();
        let delta = self.end_metres() - self.start_metres();
        if delta.length_squared() <= f32::EPSILON {
            return false;
        }
        let fraction = (point - self.start_metres()).dot(delta) / delta.length_squared();
        (0.0..=1.0).contains(&fraction)
            && point.distance(self.start_metres() + delta * fraction) <= Self::JOIN_TOLERANCE_METRES
    }
}
/// The admitted packing translation keeps its scene frame; the first hook follows
/// the original street tangent while every other endpoint follows its owner.
pub(in crate::city_layout) fn translate_property_access(
    access: &mut [CityAccessSegment],
    delta: PlanDisplacement,
    tangent: bevy::math::Dir2,
) -> GeometryResult<()> {
    for (index, segment) in access.iter_mut().enumerate() {
        let start_delta = if index == 0 {
            PlanDisplacement::try_from(*tangent * delta.metres().dot(*tangent))?
        } else {
            delta
        };
        *segment = segment.translated(start_delta, delta)?;
    }
    Ok(())
}
