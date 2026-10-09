//! Semantic map commands. Pixel displacements enter only at the canvas port.
use adventuresim_building_generator::spatial_geometry::{GeometryError, Radians};
use adventuresim_tactical_core::{
    regional_map::{MapOverlay, MapScaleError, MapSpan, MapZoomRatio},
    regional_terrain::RegionalTerrain,
    scene_input::SourcePackageDigest,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

pub(crate) use crate::strategic_scene::protocol::CanvasRect;

const MAX_BROWSER_OVERLAY_REVISION: u64 = (1_u64 << 53) - 1;

/// Document-local overlay publication ordinal, exactly representable by JS.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "u64")]
pub(crate) struct MapOverlayRevision(u64);

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
    FrameRoute,
    Hide,
    InstallTerrain {
        terrain: Box<RegionalTerrain>,
    },
    InstallOverlay {
        revision: MapOverlayRevision,
        overlay: MapOverlay,
    },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MapProtocolError {
    #[error(transparent)]
    Scale(#[from] MapScaleError),
    #[error("map pointer displacement must be finite")]
    Pointer,
    #[error("map canvas rectangle is invalid")]
    Viewport,
    #[error("map camera origin is outside WGS84")]
    Origin,
    #[error("map overlay revision must be a positive exact browser integer")]
    OverlayRevision,
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}

impl TryFrom<u64> for MapOverlayRevision {
    type Error = MapProtocolError;
    fn try_from(revision: u64) -> Result<Self, Self::Error> {
        (1..=MAX_BROWSER_OVERLAY_REVISION)
            .contains(&revision)
            .then_some(Self(revision))
            .ok_or(MapProtocolError::OverlayRevision)
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
