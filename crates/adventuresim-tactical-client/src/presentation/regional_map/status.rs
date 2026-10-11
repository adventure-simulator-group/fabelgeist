//! Small geographic status boundary; no city or terrain vertex serialization.
use super::{
    MapState, RegionalMapCamera, markers,
    protocol::{CanvasRect, MapOverlayRevision, MapProtocolError},
    surface::SurfaceCoverage,
};
use adventuresim_tactical_core::{
    regional_map::{MapScaleError, MapSpan},
    regional_terrain::RegionalTerrainRequest,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use adventuresim_world_schema::source_package::SourcePackageDigest;
use bevy::prelude::*;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};

static STATUS: OnceLock<Mutex<String>> = OnceLock::new();
const SETTLED_RENDER_FRAMES: usize = 4;

#[derive(Serialize)]
struct Status<'a> {
    ready: bool,
    presentation_ready: bool,
    overlay_revision: Option<MapOverlayRevision>,
    rect: Option<CanvasRect>,
    visible: bool,
    covered: bool,
    environment_ready: bool,
    source: Option<&'a SourcePackageDigest>,
    home: Option<Wgs84CoordinateMicrodegrees>,
    origin: Option<Wgs84CoordinateMicrodegrees>,
    span: Option<MapSpan>,
    yaw: Option<adventuresim_building_generator::spatial_geometry::Radians>,
    requested: Option<RegionalTerrainRequest>,
    presented: Option<RegionalTerrainRequest>,
    connection_meshes: usize,
    waiting_pipelines: usize,
    error: Option<Failure>,
    markers: Vec<markers::ProjectedMarker<'a>>,
    city_requested: Option<&'a adventuresim_core::strategic_place::StrategicPlaceId>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum Failure {
    Span,
    Zoom,
    Pointer,
    Viewport,
    Origin,
    OverlayRevision,
    RouteCapacity,
    ConnectionCapacity,
    Geometry,
}

impl From<&MapProtocolError> for Failure {
    fn from(error: &MapProtocolError) -> Self {
        match error {
            MapProtocolError::Scale(MapScaleError::Span { .. }) => Self::Span,
            MapProtocolError::Scale(MapScaleError::Zoom { .. }) => Self::Zoom,
            MapProtocolError::Scale(MapScaleError::Geometry(_)) => Self::Geometry,
            MapProtocolError::Pointer => Self::Pointer,
            MapProtocolError::Viewport => Self::Viewport,
            MapProtocolError::Origin => Self::Origin,
            MapProtocolError::OverlayRevision => Self::OverlayRevision,
            MapProtocolError::Geometry(_) => Self::Geometry,
        }
    }
}

impl MapState {
    fn requested_window(&mut self) -> Option<RegionalTerrainRequest> {
        match self
            .pose
            .as_ref()
            .map(|pose| pose.requested_window())
            .transpose()
        {
            Ok(requested) => requested.flatten(),
            Err(error) => {
                self.failure = Some(error);
                None
            }
        }
    }
}

pub(super) fn publish(
    mut state: ResMut<MapState>,
    sky: Res<crate::presentation::AtmosphereIblCache>,
    settings: Res<crate::presentation::TacticalGraphicsSettings>,
    cameras: Query<(&Camera, &GlobalTransform), With<RegionalMapCamera>>,
) {
    let requested = state.requested_window();
    let visible = state.pose.as_ref().is_some_and(|pose| pose.rect.is_some());
    let matched = state
        .pose
        .as_ref()
        .zip(state.presented.as_ref())
        .is_some_and(|(pose, surface)| surface.source == pose.source);
    let covered = matched
        && state
            .presented
            .as_ref()
            .is_some_and(|surface| matches!(surface.coverage, SurfaceCoverage::Drawn { .. }));
    let waiting = crate::strategic_scene::status::waiting_pipelines();
    let environment_ready = sky.is_ready(&settings);
    let route_capacity_exceeded = state
        .presented_route
        .as_ref()
        .is_some_and(|route| route.capacity_exceeded());
    let connection_capacity_exceeded = state
        .presented_connections
        .as_ref()
        .is_some_and(|connections| connections.capacity_exceeded());
    if visible
        && matched
        && viewport_matches(&state, &cameras)
        && environment_ready
        && waiting == 0
        && state.failure.is_none()
    {
        state.settled_frames = state
            .settled_frames
            .saturating_add(1)
            .min(SETTLED_RENDER_FRAMES);
    } else {
        state.settled_frames = 0;
    }
    let pose = state.pose.as_ref();
    let status = Status {
        ready: state.settled_frames >= SETTLED_RENDER_FRAMES
            && !route_capacity_exceeded
            && !connection_capacity_exceeded,
        presentation_ready: state.settled_frames >= SETTLED_RENDER_FRAMES,
        overlay_revision: state.overlay_revision,
        rect: pose.and_then(|pose| pose.rect),
        visible,
        covered,
        environment_ready,
        source: pose.map(|pose| &pose.source),
        home: pose.map(|pose| pose.home),
        origin: pose.and_then(|pose| pose.geographic_origin().ok()),
        span: pose.map(|pose| pose.span),
        yaw: pose.map(|pose| pose.yaw),
        requested,
        presented: state
            .presented
            .as_ref()
            .filter(|_| matched)
            .map(|surface| surface.request),
        connection_meshes: state
            .presented_connections
            .as_ref()
            .map_or(0, |connections| connections.mesh_count()),
        waiting_pipelines: waiting,
        error: state
            .failure
            .as_ref()
            .map(Failure::from)
            .or(route_capacity_exceeded.then_some(Failure::RouteCapacity))
            .or(connection_capacity_exceeded.then_some(Failure::ConnectionCapacity)),
        markers: markers::project(&state, &cameras),
        city_requested: state.requested_city(),
    };
    match serde_json::to_string(&status) {
        Ok(json) => match STATUS.get_or_init(|| Mutex::new(String::new())).lock() {
            Ok(mut snapshot) => *snapshot = json,
            Err(error) => warn!(%error, "regional map status is unavailable"),
        },
        Err(error) => warn!(%error, "regional map status could not serialize"),
    }
}

pub(crate) fn json() -> String {
    STATUS
        .get_or_init(|| Mutex::new("{\"ready\":false,\"visible\":false}".into()))
        .lock()
        .map(|snapshot| snapshot.clone())
        .unwrap_or_else(|_| "{\"ready\":false,\"error\":\"status-unavailable\"}".into())
}

/// Bevy may resize a viewport while processing a display scale-factor event.
/// Wait for the actual camera to match the browser's physical-pixel rectangle.
fn viewport_matches(
    state: &MapState,
    cameras: &Query<(&Camera, &GlobalTransform), With<RegionalMapCamera>>,
) -> bool {
    let Some(rect) = state.pose.as_ref().and_then(|pose| pose.rect) else {
        return false;
    };
    let Ok((camera, _)) = cameras.single() else {
        return false;
    };
    camera.physical_viewport_rect().is_some_and(|viewport| {
        viewport.min == UVec2::new(rect.x, rect.y)
            && viewport.size() == UVec2::new(rect.width, rect.height)
    })
}
