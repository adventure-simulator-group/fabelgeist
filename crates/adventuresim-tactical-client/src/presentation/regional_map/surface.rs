//! One geographic surface, sharing the existing environment material and assets.
use super::{
    MapState, RegionalMapCamera, RegionalMapRoot, camera::MAP_PITCH_RADIANS, geographic_surface,
};
use crate::presentation::{
    ActiveTacticalScene, SceneEnvironment,
    ground_scatter::grass_terminal_pigment,
    vista::{TacticalVistaMaterial, regional::regional_mesh, vista_material},
};
use adventuresim_core::weather::WeatherSnapshot;
use adventuresim_tactical_core::regional_terrain::{RegionalTerrain, RegionalTerrainRequest};
use adventuresim_world_schema::source_package::SourcePackageDigest;
use adventuresim_world_schema::{
    ElevationMeters, coordinates::terrain_projection::NativeTerrainCoordinate,
};
use bevy::{camera::visibility::RenderLayers, prelude::*};

const CAMERA_STANDOFF_SPANS: f32 = 2.0;
const CAMERA_CLIP_MARGIN: f32 = 4.0;

pub(super) struct PresentedSurface {
    pub source: SourcePackageDigest,
    pub request: RegionalTerrainRequest,
    weather: WeatherSnapshot,
    grass: Color,
    pub coverage: SurfaceCoverage,
}

pub(super) enum SurfaceCoverage {
    Empty,
    Drawn {
        entity: Entity,
        mesh: Handle<Mesh>,
        material: Handle<TacticalVistaMaterial>,
        datum: ElevationMeters,
        minimum: ElevationMeters,
        maximum: ElevationMeters,
    },
}

struct ElevationBounds {
    minimum: ElevationMeters,
    maximum: ElevationMeters,
}

impl ElevationBounds {
    fn from_terrain(terrain: &RegionalTerrain) -> Option<Self> {
        let mut elevations = terrain
            .vertices()
            .iter()
            .flatten()
            .map(|vertex| vertex.elevation);
        let first = elevations.next()?;
        let mut bounds = Self {
            minimum: first,
            maximum: first,
        };
        for elevation in elevations {
            if elevation.get() < bounds.minimum.get() {
                bounds.minimum = elevation;
            }
            if elevation.get() > bounds.maximum.get() {
                bounds.maximum = elevation;
            }
        }
        Some(bounds)
    }
}

pub(super) fn present(
    mut commands: Commands,
    mut state: ResMut<MapState>,
    active: Res<ActiveTacticalScene>,
    environments: Query<&SceneEnvironment>,
    roots: Query<Entity, With<RegionalMapRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TacticalVistaMaterial>>,
) {
    let Some(pose) = &state.pose else {
        return;
    };
    if pose.rect.is_none() {
        return;
    }
    let Some(terrain) = &state.terrain else {
        return;
    };
    let Some(environment) = active
        .entity
        .and_then(|entity| environments.get(entity).ok())
    else {
        return;
    };
    let grass = grass_terminal_pigment(environment);
    if state.presented.as_ref().is_some_and(|surface| {
        surface.source == *terrain.source()
            && surface.request == terrain.request()
            && surface.weather == environment.weather
            && surface.grass == grass
    }) {
        return;
    }
    let Ok(root) = roots.single() else {
        return;
    };
    let mesh = regional_mesh(terrain, environment.weather);
    let surface = PresentedSurface {
        source: terrain.source().clone(),
        request: terrain.request(),
        weather: environment.weather,
        grass,
        coverage: match mesh {
            Some(mesh) => {
                let Some(bounds) = ElevationBounds::from_terrain(terrain) else {
                    return;
                };
                let datum = terrain.vertices()[terrain.vertices().len() / 2]
                    .map_or(bounds.minimum, |vertex| vertex.elevation);
                let mesh = meshes.add(mesh);
                let material = materials.add(vista_material(environment.weather, grass));
                let entity = commands
                    .spawn((
                        Name::new("Regional environment surface"),
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::default(),
                        ChildOf(root),
                        RenderLayers::layer(crate::strategic_scene::protocol::REGIONAL_MAP_LAYER),
                    ))
                    .id();
                SurfaceCoverage::Drawn {
                    entity,
                    mesh,
                    material,
                    datum,
                    minimum: bounds.minimum,
                    maximum: bounds.maximum,
                }
            }
            None => SurfaceCoverage::Empty,
        },
    };
    if let Some(old) = state.presented.replace(surface)
        && let SurfaceCoverage::Drawn {
            entity,
            mesh,
            material,
            ..
        } = old.coverage
    {
        commands.entity(entity).despawn();
        meshes.remove(mesh.id());
        materials.remove(material.id());
    }
    state.settled_frames = 0;
}

pub(super) fn sync_camera(
    state: Res<MapState>,
    windows: Query<&Window>,
    mut roots: Query<&mut Visibility, With<RegionalMapRoot>>,
    mut cameras: Query<(&mut Camera, &mut Transform, &mut Projection), With<RegionalMapCamera>>,
) {
    let Ok((mut camera, mut transform, mut projection)) = cameras.single_mut() else {
        return;
    };
    camera.is_active = false;
    for mut root in &mut roots {
        *root = Visibility::Hidden;
    }
    let (Some(pose), Some(surface)) = (&state.pose, &state.presented) else {
        return;
    };
    let Some(rect) = pose.rect else {
        return;
    };
    if surface.source != pose.source {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    rect.apply(&mut camera, window.resolution.physical_size());
    if !camera.is_active {
        return;
    }
    // An empty admitted window still clears its viewport. Deactivating the
    // last camera could leave the previous frame painted on the canvas.
    let SurfaceCoverage::Drawn {
        datum,
        minimum,
        maximum,
        ..
    } = surface.coverage
    else {
        return;
    };
    let offset =
        NativeTerrainCoordinate::from(surface.request.origin.to_e7()).offset_to(pose.origin);
    // Only the camera may retain the window datum over a coverage hole. Covered
    // focus positions use the same interpolated source plane as pins and routes.
    let elevation = state
        .terrain
        .as_ref()
        .filter(|terrain| terrain.request() == surface.request)
        .and_then(|terrain| geographic_surface::position_at_offset(terrain, offset))
        .map_or(f32::from(datum.get()), |point| point.y);
    let target = Vec3::new(
        offset.east_metres as f32,
        elevation,
        -offset.north_metres as f32,
    );
    let elevation_range = f32::from(maximum.get()) - f32::from(minimum.get());
    let distance = pose.span.metres().max(elevation_range) * CAMERA_STANDOFF_SPANS;
    let yaw = pose.yaw.radians();
    let direction = Vec3::new(
        yaw.sin() * MAP_PITCH_RADIANS.cos(),
        MAP_PITCH_RADIANS.sin(),
        yaw.cos() * MAP_PITCH_RADIANS.cos(),
    );
    *transform =
        Transform::from_translation(target + direction * distance).looking_at(target, Vec3::Y);
    *projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::FixedVertical {
            viewport_height: pose.span.metres(),
        },
        near: 0.0,
        far: distance * CAMERA_CLIP_MARGIN,
        ..OrthographicProjection::default_3d()
    });
    for mut root in &mut roots {
        *root = Visibility::Inherited;
    }
}
