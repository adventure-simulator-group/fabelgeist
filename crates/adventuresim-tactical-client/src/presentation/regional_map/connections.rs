//! Retained source connection meshes, sharing the terrain triangle kernel.
use super::{
    MapState, RegionalMapRoot,
    path_geometry::{GeographicLine, MAX_PATH_MESH_VERTICES, PathMeshBuilder},
};
use adventuresim_tactical_core::{
    regional_environment::RegionalEnvironment, regional_terrain::RegionalTerrainRequest,
};
use adventuresim_world_schema::{
    regional_connection::RegionalConnectionKind, source_package::SourcePackageDigest,
};
use bevy::{camera::visibility::RenderLayers, prelude::*};

const CONNECTION_WIDTH_PHYSICAL_PIXELS: f32 = 2.0;
const CONNECTION_KINDS: [RegionalConnectionKind; 7] = [
    RegionalConnectionKind::Land,
    RegionalConnectionKind::River,
    RegionalConnectionKind::Coast,
    RegionalConnectionKind::Canal,
    RegionalConnectionKind::Ferry,
    RegionalConnectionKind::Winter,
    RegionalConnectionKind::InferredWalkingLink,
];

pub(super) struct PresentedConnections {
    source: SourcePackageDigest,
    window: RegionalTerrainRequest,
    width_metres: f32,
    coverage: ConnectionCoverage,
}

struct ConnectionAssets {
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

struct ClassifiedMesh {
    kind: RegionalConnectionKind,
    mesh: Mesh,
}

enum ConnectionCoverage {
    Drawn(Vec<ConnectionAssets>),
    CapacityExceeded,
}

impl PresentedConnections {
    pub(super) fn mesh_count(&self) -> usize {
        match &self.coverage {
            ConnectionCoverage::Drawn(assets) => assets.len(),
            ConnectionCoverage::CapacityExceeded => 0,
        }
    }

    pub(super) fn capacity_exceeded(&self) -> bool {
        matches!(self.coverage, ConnectionCoverage::CapacityExceeded)
    }
}

pub(super) fn present(
    mut commands: Commands,
    mut state: ResMut<MapState>,
    roots: Query<Entity, With<RegionalMapRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(pose) = &state.pose else {
        return;
    };
    let Some(rect) = pose.rect else {
        return;
    };
    let Some(environment) = state
        .environment
        .as_ref()
        .filter(|environment| environment.terrain().source() == &pose.source)
        .filter(|environment| {
            state
                .presented
                .as_ref()
                .is_some_and(|surface| surface.request == environment.terrain().request())
        })
    else {
        remove(
            state.presented_connections.take(),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
        return;
    };
    let width_metres =
        pose.span.metres() / rect.full_height as f32 * CONNECTION_WIDTH_PHYSICAL_PIXELS;
    if state
        .presented_connections
        .as_ref()
        .is_some_and(|presented| {
            presented.source == pose.source
                && presented.window == environment.terrain().request()
                && presented.width_metres == width_metres
        })
    {
        return;
    }
    let Ok(root) = roots.single() else {
        return;
    };
    let coverage = match geometry(environment, width_metres) {
        Ok(geometry) => ConnectionCoverage::Drawn(
            geometry
                .into_iter()
                .map(|geometry| {
                    let material = materials.add(StandardMaterial {
                        base_color: color(geometry.kind),
                        unlit: true,
                        ..default()
                    });
                    let mesh = meshes.add(geometry.mesh);
                    let entity = commands
                        .spawn((
                            Name::new("Regional geographic connections"),
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::default(),
                            ChildOf(root),
                            RenderLayers::layer(
                                crate::strategic_scene::protocol::REGIONAL_MAP_LAYER,
                            ),
                        ))
                        .id();
                    ConnectionAssets {
                        entity,
                        mesh,
                        material,
                    }
                })
                .collect(),
        ),
        Err(_) => ConnectionCoverage::CapacityExceeded,
    };
    let presented = PresentedConnections {
        source: pose.source.clone(),
        window: environment.terrain().request(),
        width_metres,
        coverage,
    };
    let old = state.presented_connections.replace(presented);
    remove(old, &mut commands, &mut meshes, &mut materials);
    state.settled_frames = 0;
}

fn geometry(
    environment: &RegionalEnvironment,
    width_metres: f32,
) -> Result<Vec<ClassifiedMesh>, super::path_geometry::PathGeometryError> {
    let mut geometry = Vec::new();
    let mut vertices = 0;
    for kind in CONNECTION_KINDS {
        let mut builder = PathMeshBuilder::default();
        for line in environment
            .connections()
            .iter()
            .filter(|line| line.kind() == kind)
        {
            builder.add(
                environment.terrain(),
                GeographicLine::Connection(line),
                width_metres,
            )?;
        }
        vertices += builder.vertex_count();
        if vertices > MAX_PATH_MESH_VERTICES {
            return Err(super::path_geometry::PathGeometryError::CapacityExceeded);
        }
        if let Some(mesh) = builder.finish() {
            geometry.push(ClassifiedMesh { kind, mesh });
        }
    }
    Ok(geometry)
}

fn color(kind: RegionalConnectionKind) -> Color {
    let channels = match kind {
        RegionalConnectionKind::Land => [77_u8, 49, 28],
        RegionalConnectionKind::River
        | RegionalConnectionKind::Coast
        | RegionalConnectionKind::Canal => [61, 137, 190],
        RegionalConnectionKind::Ferry => [135, 192, 226],
        RegionalConnectionKind::Winter => [214, 220, 225],
        RegionalConnectionKind::InferredWalkingLink => [119, 101, 76],
    }
    .map(|channel| f32::from(channel) / 255.0);
    Color::srgb(channels[0], channels[1], channels[2])
}

fn remove(
    presented: Option<PresentedConnections>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    if let Some(PresentedConnections {
        coverage: ConnectionCoverage::Drawn(assets),
        ..
    }) = presented
    {
        for assets in assets {
            commands.entity(assets.entity).despawn();
            meshes.remove(assets.mesh.id());
            materials.remove(assets.material.id());
        }
    }
}
