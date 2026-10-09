//! Geographic presentation retained alongside city and tactical scenes.
//! This owns no strategic authority, collision or replicated tactical state.
use super::RegionalMapCamera;
use adventuresim_tactical_core::regional_terrain::RegionalTerrain;
use bevy::prelude::*;
use camera::MapPose;
use protocol::{MapCommand, MapProtocolError};

mod camera;
mod lighting;
pub(crate) mod protocol;
pub(crate) mod status;
mod surface;

const MAP_CAMERA_ORDER: isize = 64;

#[derive(Component)]
struct RegionalMapRoot;

#[derive(Resource, Default)]
struct MapState {
    pose: Option<MapPose>,
    terrain: Option<Box<RegionalTerrain>>,
    presented: Option<surface::PresentedSurface>,
    failure: Option<MapProtocolError>,
    settled_frames: usize,
}

pub(crate) struct RegionalMapPlugin;

impl Plugin for RegionalMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapState>()
            .add_systems(Startup, setup)
            .add_systems(Update, (surface::present, lighting::sync))
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
                if let Some(pose) = state.pose.as_mut()
                    && pose.source == source
                    && pose.home == origin
                {
                    pose.rect = Some(rect);
                } else {
                    state.pose = Some(MapPose::new(source, origin, span, rect));
                    state.terrain = None;
                }
                Ok(())
            }
            Self::Resize { rect } => {
                if let Some(pose) = state.pose.as_mut() {
                    pose.rect = Some(rect);
                }
                Ok(())
            }
            Self::Pan { delta } => state.pose.as_mut().map_or(Ok(()), |pose| pose.pan(delta)),
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
            Self::Hide => {
                if let Some(pose) = state.pose.as_mut() {
                    pose.rect = None;
                }
                Ok(())
            }
            Self::InstallTerrain { terrain } => {
                if state
                    .pose
                    .as_ref()
                    .is_some_and(|pose| pose.source == *terrain.source())
                {
                    state.terrain = Some(terrain);
                }
                Ok(())
            }
        };
        state.failure = result.err();
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
