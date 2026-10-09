//! A retained, bounded selected-route mesh beside the geographic terrain.
use super::{MapState, RegionalMapRoot, path_geometry};
use adventuresim_tactical_core::{
    regional_map::{MapRoute, MapRouteKind},
    regional_terrain::RegionalTerrainRequest,
};
use adventuresim_world_schema::source_package::SourcePackageDigest;
use bevy::{camera::visibility::RenderLayers, prelude::*};

const ROUTE_WIDTH_PHYSICAL_PIXELS: f32 = 3.0;
const COMPUTED_ROUTE_SRGB: [u8; 3] = [244, 195, 96];
const ESTIMATED_ROUTE_SRGB: [u8; 3] = [207, 201, 182];

pub(super) struct PresentedRoute {
    source: SourcePackageDigest,
    window: RegionalTerrainRequest,
    route: MapRoute,
    /// Bevy mesh adapter width, in metres at the current physical-pixel scale.
    width_metres: f32,
    coverage: RouteCoverage,
}

struct RouteAssets {
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

enum RouteCoverage {
    Drawn(RouteAssets),
    Uncovered,
    CapacityExceeded,
}

impl PresentedRoute {
    pub(super) fn capacity_exceeded(&self) -> bool {
        matches!(self.coverage, RouteCoverage::CapacityExceeded)
    }
}

impl RouteAssets {
    fn install(
        root: Entity,
        mesh: Mesh,
        kind: MapRouteKind,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        let mesh = meshes.add(mesh);
        let rgb = match kind {
            MapRouteKind::Computed => COMPUTED_ROUTE_SRGB,
            MapRouteKind::Estimate => ESTIMATED_ROUTE_SRGB,
        }
        .map(|channel| f32::from(channel) / 255.0);
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            unlit: true,
            ..default()
        });
        let entity = commands
            .spawn((
                Name::new("Selected geographic route"),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                ChildOf(root),
                RenderLayers::layer(crate::strategic_scene::protocol::REGIONAL_MAP_LAYER),
            ))
            .id();
        Self {
            entity,
            mesh,
            material,
        }
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
    let admitted = state
        .overlay
        .as_ref()
        .filter(|overlay| overlay.source() == &pose.source)
        .and_then(|overlay| overlay.route());
    let terrain = state
        .terrain()
        .filter(|terrain| terrain.source() == &pose.source)
        .filter(|terrain| {
            state
                .presented
                .as_ref()
                .is_some_and(|surface| surface.request == terrain.request())
        });
    let (Some(route), Some(terrain)) = (admitted, terrain) else {
        remove(
            state.presented_route.take(),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
        return;
    };
    let width_metres = pose.span.metres() / rect.full_height as f32 * ROUTE_WIDTH_PHYSICAL_PIXELS;
    if state.presented_route.as_ref().is_some_and(|presented| {
        presented.source == pose.source
            && presented.window == terrain.request()
            && presented.route == *route
            && presented.width_metres == width_metres
    }) {
        return;
    }
    let Ok(root) = roots.single() else {
        return;
    };
    let coverage = match path_geometry::mesh(
        terrain,
        path_geometry::GeographicLine::Route(route),
        width_metres,
    ) {
        Ok(Some(mesh)) => RouteCoverage::Drawn(RouteAssets::install(
            root,
            mesh,
            route.kind(),
            &mut commands,
            &mut meshes,
            &mut materials,
        )),
        Ok(None) => RouteCoverage::Uncovered,
        Err(_) => RouteCoverage::CapacityExceeded,
    };
    let source = pose.source.clone();
    let window = terrain.request();
    let route = route.clone();
    let old = state.presented_route.replace(PresentedRoute {
        source,
        window,
        route,
        width_metres,
        coverage,
    });
    remove(old, &mut commands, &mut meshes, &mut materials);
    state.settled_frames = 0;
}

fn remove(
    presented: Option<PresentedRoute>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    if let Some(PresentedRoute {
        coverage: RouteCoverage::Drawn(assets),
        ..
    }) = presented
    {
        commands.entity(assets.entity).despawn();
        meshes.remove(assets.mesh.id());
        materials.remove(assets.material.id());
    }
}
