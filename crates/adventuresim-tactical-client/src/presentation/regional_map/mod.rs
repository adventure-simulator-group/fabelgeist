//! Geographic presentation retained alongside city and tactical scenes.
//! This owns no strategic authority, collision or replicated tactical state.
use super::RegionalMapCamera;
use adventuresim_tactical_core::regional_map::{MapOverlay, MapSpan};
use adventuresim_tactical_core::{
    regional_environment::RegionalEnvironment, regional_terrain::RegionalTerrain,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use adventuresim_world_schema::source_package::SourcePackageDigest;
use bevy::prelude::*;
use camera::MapPose;
use protocol::{MapCommand, MapOverlayRevision, MapProtocolError};

mod camera;
mod connections;
mod geographic_surface;
mod lighting;
mod markers;
mod path_geometry;
pub(crate) mod protocol;
mod routes;
pub(crate) mod status;
mod surface;

const MAP_CAMERA_ORDER: isize = 64;

#[derive(Component)]
struct RegionalMapRoot;

#[derive(Resource, Default)]
struct MapState {
    pose: Option<MapPose>,
    environment: Option<Box<RegionalEnvironment>>,
    presented_connections: Option<connections::PresentedConnections>,
    presented: Option<surface::PresentedSurface>,
    overlay: Option<MapOverlay>,
    overlay_revision: Option<MapOverlayRevision>,
    presented_route: Option<routes::PresentedRoute>,
    failure: Option<MapProtocolError>,
    settled_frames: usize,
}

pub(crate) struct RegionalMapPlugin;

impl Plugin for RegionalMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapState>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (surface::present, connections::present, routes::present).chain(),
            )
            .add_systems(Update, lighting::sync)
            .add_systems(
                PostUpdate,
                surface::sync_camera
                    .before(bevy::camera::CameraUpdateSystems)
                    .before(bevy::transform::TransformSystems::Propagate),
            )
            .add_systems(Last, status::publish);
    }
}

impl MapCommand {
    pub(crate) fn apply(self, world: &mut World) {
        let mut state = world.resource_mut::<MapState>();
        let result = match self {
            Self::Open {
                source,
                origin,
                span,
                rect,
            } => {
                state.open(source, origin, span, rect);
                Ok(())
            }
            Self::Resize { rect } => {
                if let Some(pose) = state.pose.as_mut()
                    && pose.rect != Some(rect)
                {
                    pose.rect = Some(rect);
                    state.settled_frames = 0;
                }
                Ok(())
            }
            Self::Pan { delta } => state.pan(delta),
            Self::Zoom { ratio } => state.pose.as_mut().map_or(Ok(()), |pose| {
                if pose.rect.is_some() {
                    pose.zoom(ratio)
                } else {
                    Ok(())
                }
            }),
            Self::Rotate { angle } => state.pose.as_mut().map_or(Ok(()), |pose| {
                if pose.rect.is_some() {
                    pose.rotate(angle)
                } else {
                    Ok(())
                }
            }),
            Self::Reset => {
                if let Some(pose) = state.pose.as_mut() {
                    pose.reset();
                }
                Ok(())
            }
            Self::FrameRoute => state.frame_route(),
            Self::Hide => {
                if let Some(pose) = state.pose.as_mut() {
                    pose.rect = None;
                }
                Ok(())
            }
            Self::InstallEnvironment { environment } => {
                if state
                    .pose
                    .as_ref()
                    .is_some_and(|pose| pose.source == *environment.terrain().source())
                {
                    state.environment = Some(environment);
                }
                Ok(())
            }
            Self::InstallOverlay { revision, overlay } => {
                if state
                    .pose
                    .as_ref()
                    .is_some_and(|pose| &pose.source == overlay.source())
                    && state
                        .overlay_revision
                        .is_none_or(|current| revision > current)
                {
                    state.overlay = Some(overlay);
                    state.overlay_revision = Some(revision);
                    state.settled_frames = 0;
                }
                Ok(())
            }
        };
        state.failure = result.err();
    }
}

impl MapState {
    fn terrain(&self) -> Option<&RegionalTerrain> {
        self.environment
            .as_ref()
            .map(|environment| environment.terrain())
    }

    fn open(
        &mut self,
        source: SourcePackageDigest,
        origin: Wgs84CoordinateMicrodegrees,
        span: MapSpan,
        rect: protocol::CanvasRect,
    ) {
        if let Some(pose) = self.pose.as_mut()
            && pose.source == source
            && pose.home == origin
        {
            pose.rect = Some(rect);
        } else {
            self.pose = Some(MapPose::new(source, origin, span, rect));
            self.environment = None;
            self.overlay = None;
            self.overlay_revision = None;
        }
    }

    fn pan(&mut self, delta: protocol::MapPointerDisplacement) -> camera::Result<()> {
        let Some(pose) = self.pose.as_mut() else {
            return Ok(());
        };
        let frame = self
            .presented
            .as_ref()
            .filter(|surface| surface.source == pose.source)
            .map_or(pose.home, |surface| surface.request.origin);
        pose.pan(delta, frame)
    }

    fn frame_route(&mut self) -> camera::Result<()> {
        match (
            self.pose.as_mut(),
            self.overlay.as_ref().and_then(MapOverlay::route),
        ) {
            (Some(pose), Some(route)) => pose.frame_route(route),
            _ => Ok(()),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((RegionalMapRoot, Transform::default(), Visibility::Hidden));
    commands.spawn((
        RegionalMapCamera,
        crate::strategic_scene::StrategicCamera,
        crate::presentation::interior_lighting::FixedViewExposure,
        Camera3d::default(),
        Camera {
            order: MAP_CAMERA_ORDER,
            is_active: false,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.13, 0.16, 0.19)),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection::default_3d()),
        bevy::camera::visibility::RenderLayers::layer(
            crate::strategic_scene::protocol::REGIONAL_MAP_LAYER,
        ),
        bevy::camera::Exposure::SUNLIGHT,
        Msaa::Off,
    ));
}
