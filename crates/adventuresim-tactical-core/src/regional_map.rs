//! Bounded, observer-admitted geographic overlays for environment presentation.
//! Producers admit knowledge; this contract carries no discovery authority.
use crate::scene_input::SourcePackageDigest;
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength};
use adventuresim_core::strategic_place::StrategicPlaceId;
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_MAP_MARKERS: usize = 32_768;
pub const MAX_MAP_ROUTE_POINTS: usize = 65_536;
pub const MIN_MAP_SPAN_METRES: f32 = 40.0;
pub const MAX_MAP_SPAN_METRES: f32 = 8_192_000.0;
const MIN_ZOOM_RATIO: f32 = 0.05;
const MAX_ZOOM_RATIO: f32 = 20.0;
pub type Result<T> = std::result::Result<T, MapOverlayError>;

/// Checked vertical extent of the orthographic geographic view.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "PositiveLength")]
pub struct MapSpan(PositiveLength);

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(try_from = "f32")]
pub struct MapZoomRatio(f32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapMarker {
    pub place: StrategicPlaceId,
    pub origin: Wgs84CoordinateMicrodegrees,
    pub rank: MapMarkerRank,
    pub emphasis: MapMarkerEmphasis,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MapOverlayWire", deny_unknown_fields)]
pub struct MapOverlay {
    source: SourcePackageDigest,
    markers: Vec<MapMarker>,
    route: Option<MapRoute>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MapOverlayWire {
    source: SourcePackageDigest,
    markers: Vec<MapMarker>,
    route: Option<MapRoute>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MapRouteWire", deny_unknown_fields)]
pub struct MapRoute {
    kind: MapRouteKind,
    points: Vec<Wgs84CoordinateMicrodegrees>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MapRouteWire {
    kind: MapRouteKind,
    points: Vec<Wgs84CoordinateMicrodegrees>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MapMarkerRank {
    Hamlet,
    Village,
    Town,
    City,
    Capital,
    CaseSite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MapMarkerEmphasis {
    Ordinary,
    Connected,
    Current,
    Selected,
    CurrentSelected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MapRouteKind {
    Computed,
    Estimate,
}

#[derive(Debug, thiserror::Error)]
pub enum MapOverlayError {
    #[error("map has {provided} markers; limit is {MAX_MAP_MARKERS}")]
    MarkerCount { provided: usize },
    #[error("map marker must name a settlement or an exact case site with the matching rank")]
    MarkerPlace,
    #[error("map repeats a marker place")]
    DuplicateMarker,
    #[error("map has more than one current place")]
    DuplicateCurrent,
    #[error("map has more than one selected place")]
    DuplicateSelection,
    #[error("map route has {provided} points; expected 2 through {MAX_MAP_ROUTE_POINTS}")]
    RoutePointCount { provided: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum MapScaleError {
    #[error("map span {metres} metres is outside the presentation range")]
    Span { metres: f32 },
    #[error("map zoom ratio {ratio} is outside the presentation range")]
    Zoom { ratio: f32 },
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}

impl MapSpan {
    /// Native orthographic camera adapter, in vertical scene metres.
    pub fn metres(self) -> f32 {
        self.0.metres()
    }

    pub fn zoomed(self, ratio: MapZoomRatio) -> std::result::Result<Self, MapScaleError> {
        let metres = (f64::from(self.metres()) * f64::from(ratio.0)).clamp(
            f64::from(MIN_MAP_SPAN_METRES),
            f64::from(MAX_MAP_SPAN_METRES),
        ) as f32;
        Self::try_from(PositiveLength::from_metres(metres)?)
    }
}

impl TryFrom<PositiveLength> for MapSpan {
    type Error = MapScaleError;
    fn try_from(length: PositiveLength) -> std::result::Result<Self, Self::Error> {
        let metres = length.metres();
        if !(MIN_MAP_SPAN_METRES..=MAX_MAP_SPAN_METRES).contains(&metres) {
            return Err(MapScaleError::Span { metres });
        }
        Ok(Self(length))
    }
}

impl TryFrom<f32> for MapZoomRatio {
    type Error = MapScaleError;
    fn try_from(ratio: f32) -> std::result::Result<Self, Self::Error> {
        if !ratio.is_finite() || !(MIN_ZOOM_RATIO..=MAX_ZOOM_RATIO).contains(&ratio) {
            return Err(MapScaleError::Zoom { ratio });
        }
        Ok(Self(ratio))
    }
}

impl MapOverlay {
    pub fn new(
        source: SourcePackageDigest,
        markers: Vec<MapMarker>,
        route: Option<MapRoute>,
    ) -> Result<Self> {
        if markers.len() > MAX_MAP_MARKERS {
            return Err(MapOverlayError::MarkerCount {
                provided: markers.len(),
            });
        }
        let mut places = BTreeSet::new();
        let mut current = false;
        let mut selected = false;
        for marker in &markers {
            match (&marker.place, marker.rank) {
                (StrategicPlaceId::CaseSite { .. }, MapMarkerRank::CaseSite) => {}
                (StrategicPlaceId::Settlement { .. }, rank) if rank != MapMarkerRank::CaseSite => {}
                _ => return Err(MapOverlayError::MarkerPlace),
            }
            if !places.insert(&marker.place) {
                return Err(MapOverlayError::DuplicateMarker);
            }
            if matches!(
                marker.emphasis,
                MapMarkerEmphasis::Current | MapMarkerEmphasis::CurrentSelected
            ) {
                if current {
                    return Err(MapOverlayError::DuplicateCurrent);
                }
                current = true;
            }
            if matches!(
                marker.emphasis,
                MapMarkerEmphasis::Selected | MapMarkerEmphasis::CurrentSelected
            ) {
                if selected {
                    return Err(MapOverlayError::DuplicateSelection);
                }
                selected = true;
            }
        }
        Ok(Self {
            source,
            markers,
            route,
        })
    }

    pub fn source(&self) -> &SourcePackageDigest {
        &self.source
    }
    pub fn markers(&self) -> &[MapMarker] {
        &self.markers
    }
    pub fn route(&self) -> Option<&MapRoute> {
        self.route.as_ref()
    }
}

impl TryFrom<MapOverlayWire> for MapOverlay {
    type Error = MapOverlayError;
    fn try_from(wire: MapOverlayWire) -> Result<Self> {
        Self::new(wire.source, wire.markers, wire.route)
    }
}

impl MapRoute {
    pub fn new(kind: MapRouteKind, points: Vec<Wgs84CoordinateMicrodegrees>) -> Result<Self> {
        if !(2..=MAX_MAP_ROUTE_POINTS).contains(&points.len()) {
            return Err(MapOverlayError::RoutePointCount {
                provided: points.len(),
            });
        }
        Ok(Self { kind, points })
    }
    pub const fn kind(&self) -> MapRouteKind {
        self.kind
    }
    pub fn points(&self) -> &[Wgs84CoordinateMicrodegrees] {
        &self.points
    }
}

impl TryFrom<MapRouteWire> for MapRoute {
    type Error = MapOverlayError;
    fn try_from(wire: MapRouteWire) -> Result<Self> {
        Self::new(wire.kind, wire.points)
    }
}
