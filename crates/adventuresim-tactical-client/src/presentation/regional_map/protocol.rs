//! Semantic map commands. Pixel displacements enter only at the canvas port.
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength, Radians};
use adventuresim_tactical_core::{
    regional_terrain::RegionalTerrain, scene_input::SourcePackageDigest,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

pub(crate) use crate::strategic_scene::protocol::CanvasRect;
pub(super) const MIN_MAP_SPAN_METRES: f32 = 40.0;
pub(super) const MAX_MAP_SPAN_METRES: f32 = 8_192_000.0;
const MIN_ZOOM_RATIO: f32 = 0.05;
const MAX_ZOOM_RATIO: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PositiveLength")]
pub(crate) struct MapSpan(PositiveLength);

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(try_from = "f32")]
pub(crate) struct MapZoomRatio(f32);

/// Physical canvas pixels, not scene metres or a geographic position.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(try_from = "Vec2")]
pub(crate) struct MapPointerDisplacement(Vec2);

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum MapCommand {
    Open {
        source: SourcePackageDigest,
        origin: Wgs84CoordinateMicrodegrees,
        span: MapSpan,
        #[serde(deserialize_with = "viewport")]
        rect: CanvasRect,
    },
    Resize {
        #[serde(deserialize_with = "viewport")]
        rect: CanvasRect,
    },
    Pan {
        delta: MapPointerDisplacement,
    },
    Zoom {
        ratio: MapZoomRatio,
    },
    Rotate {
        angle: Radians,
    },
    Reset,
    Hide,
    InstallTerrain {
        terrain: Box<RegionalTerrain>,
    },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MapProtocolError {
    #[error("map span {metres} metres is outside the presentation range")]
    Span { metres: f32 },
    #[error("map zoom ratio {ratio} is outside the presentation range")]
    Zoom { ratio: f32 },
    #[error("map pointer displacement must be finite")]
    Pointer,
    #[error("map canvas rectangle is invalid")]
    Viewport,
    #[error("map camera origin is outside WGS84")]
    Origin,
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}

impl MapSpan {
    /// Orthographic camera numerical port, in vertical scene metres.
    pub(crate) fn metres(self) -> f32 {
        self.0.metres()
    }

    pub(super) fn zoomed(self, ratio: MapZoomRatio) -> Result<Self, MapProtocolError> {
        let metres = (f64::from(self.metres()) * f64::from(ratio.0)).clamp(
            f64::from(MIN_MAP_SPAN_METRES),
            f64::from(MAX_MAP_SPAN_METRES),
        ) as f32;
        Self::try_from(PositiveLength::from_metres(metres)?)
    }
}

impl TryFrom<PositiveLength> for MapSpan {
    type Error = MapProtocolError;
    fn try_from(length: PositiveLength) -> Result<Self, Self::Error> {
        let metres = length.metres();
        if !(MIN_MAP_SPAN_METRES..=MAX_MAP_SPAN_METRES).contains(&metres) {
            return Err(MapProtocolError::Span { metres });
        }
        Ok(Self(length))
    }
}

impl TryFrom<f32> for MapZoomRatio {
    type Error = MapProtocolError;
    fn try_from(ratio: f32) -> Result<Self, Self::Error> {
        if !ratio.is_finite() || !(MIN_ZOOM_RATIO..=MAX_ZOOM_RATIO).contains(&ratio) {
            return Err(MapProtocolError::Zoom { ratio });
        }
        Ok(Self(ratio))
    }
}

impl MapPointerDisplacement {
    pub(super) fn pixels(self) -> Vec2 {
        self.0
    }
}

impl TryFrom<Vec2> for MapPointerDisplacement {
    type Error = MapProtocolError;
    fn try_from(pixels: Vec2) -> Result<Self, Self::Error> {
        pixels
            .is_finite()
            .then_some(Self(pixels))
            .ok_or(MapProtocolError::Pointer)
    }
}

/// CanvasRect is the shared native viewport port. Admit map rectangles here;
/// half-visible CSS edges can round outward by at most one physical pixel.
fn viewport<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<CanvasRect, D::Error> {
    let rect = CanvasRect::deserialize(deserializer)?;
    let valid = rect.width > 0
        && rect.height > 0
        && rect.full_width >= rect.width
        && rect.full_height >= rect.height
        && rect.offset_x.is_finite()
        && rect.offset_y.is_finite()
        && rect.offset_x >= 0.0
        && rect.offset_y >= 0.0
        && f64::from(rect.offset_x) + f64::from(rect.width) <= f64::from(rect.full_width) + 1.0
        && f64::from(rect.offset_y) + f64::from(rect.height) <= f64::from(rect.full_height) + 1.0;
    if valid {
        Ok(rect)
    } else {
        Err(serde::de::Error::custom(MapProtocolError::Viewport))
    }
}
