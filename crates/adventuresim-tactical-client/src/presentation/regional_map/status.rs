//! Small geographic status boundary; no city or terrain vertex serialization.
use super::{MapState, protocol::MapProtocolError, surface::SurfaceCoverage};
use adventuresim_tactical_core::{
    regional_terrain::RegionalTerrainRequest, scene_input::SourcePackageDigest,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use bevy::prelude::*;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};

static STATUS: OnceLock<Mutex<String>> = OnceLock::new();
const SETTLED_RENDER_FRAMES: usize = 4;

#[derive(Serialize)]
struct Status<'a> {
    ready: bool,
    visible: bool,
    covered: bool,
    environment_ready: bool,
    source: Option<&'a SourcePackageDigest>,
    home: Option<Wgs84CoordinateMicrodegrees>,
    origin: Option<Wgs84CoordinateMicrodegrees>,
    span: Option<super::protocol::MapSpan>,
    yaw: Option<adventuresim_building_generator::spatial_geometry::Radians>,
    requested: Option<RegionalTerrainRequest>,
    presented: Option<RegionalTerrainRequest>,
    waiting_pipelines: usize,
    error: Option<Failure>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum Failure {
    Span,
    Zoom,
    Pointer,
    Viewport,
    Origin,
    Geometry,
}

impl From<&MapProtocolError> for Failure {
    fn from(error: &MapProtocolError) -> Self {
        match error {
            MapProtocolError::Span { .. } => Self::Span,
            MapProtocolError::Zoom { .. } => Self::Zoom,
            MapProtocolError::Pointer => Self::Pointer,
            MapProtocolError::Viewport => Self::Viewport,
            MapProtocolError::Origin => Self::Origin,
            MapProtocolError::Geometry(_) => Self::Geometry,
        }
    }
}

pub(super) fn publish(
    mut state: ResMut<MapState>,
    sky: Res<crate::presentation::AtmosphereIblCache>,
    settings: Res<crate::presentation::TacticalGraphicsSettings>,
) {
    let requested = state
        .pose
        .as_ref()
        .map(|pose| pose.requested_window())
        .transpose();
    let requested = match requested {
        Ok(requested) => requested.flatten(),
        Err(error) => {
            state.failure = Some(error);
            None
        }
    };
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
    if visible && matched && environment_ready && waiting == 0 && state.failure.is_none() {
        state.settled_frames = state
            .settled_frames
            .saturating_add(1)
            .min(SETTLED_RENDER_FRAMES);
    } else {
        state.settled_frames = 0;
    }
    let pose = state.pose.as_ref();
    let status = Status {
        ready: state.settled_frames >= SETTLED_RENDER_FRAMES,
        visible,
        covered,
        environment_ready,
        source: pose.map(|pose| &pose.source),
        home: pose.map(|pose| pose.home),
        origin: pose.map(|pose| pose.origin),
        span: pose.map(|pose| pose.span),
        yaw: pose.map(|pose| pose.yaw),
        requested,
        presented: state
            .presented
            .as_ref()
            .filter(|_| matched)
            .map(|surface| surface.request),
        waiting_pipelines: waiting,
        error: state.failure.as_ref().map(Failure::from),
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
