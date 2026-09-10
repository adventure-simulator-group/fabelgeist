//! Shared, terrain-conforming city ground presentation at every viewing distance.

use super::*;

mod activity;
mod material;
mod mesh;
mod support;
mod traffic;
pub(in crate::presentation::vista) use support::GroundSupport;

use material::CityGroundKind;
pub(crate) use material::CityGroundMaterial;
use mesh::CitySurfaceMeshBuilder;

#[derive(bevy::ecs::system::SystemParam)]
pub(in crate::presentation) struct CityGroundAssets<'w> {
    pub(super) materials: ResMut<'w, Assets<CityGroundMaterial>>,
    pub(super) textures: Res<'w, ProceduralTextureAssets>,
}

#[derive(Clone, Copy)]
pub(super) struct UrbanGround<'a> {
    streets: &'a [CityStreetPatch],
    yards: &'a [CityYardPatch],
}

impl<'a> UrbanGround<'a> {
    pub(super) const fn new(streets: &'a [CityStreetPatch], yards: &'a [CityYardPatch]) -> Self {
        Self { streets, yards }
    }

    pub(super) fn suppresses_grass(self, point: Vec2) -> bool {
        self.streets.iter().any(|street| street.contains(point))
            || self.yards.iter().any(|yard| yard.contains(point))
    }
}

#[derive(Component)]
pub(crate) struct CityStreetPresentation;

#[derive(Component)]
pub(crate) struct CityYardPresentation;

impl CityGroundAssets<'_> {
    pub(in crate::presentation::vista) fn spawn(
        &mut self,
        commands: &mut Commands,
        bundle: &SceneVistaBundle,
        support: &GroundSupport,
        environment: &SceneEnvironment,
        meshes: &mut Assets<Mesh>,
        images: &mut Assets<Image>,
    ) {
        let streets = &bundle.streets;
        let yards = &bundle.yards;
        let groups = &bundle.furniture_groups;
        // Local footprint coordinates bound shader work independently of city size.
        let mut builders: [CitySurfaceMeshBuilder; 5] = Default::default();
        for yard in yards.iter().copied() {
            let kind = CityGroundKind::from(yard.surface);
            builders[kind.index()].append_yard(yard, support, groups);
        }
        for street in streets.iter().copied() {
            let kind = CityGroundKind::from(street.surface());
            builders[kind.index()].append_street(street, support, groups);
        }
        let network = traffic::TrafficNetwork::new(streets);
        let mut masks = std::collections::BTreeMap::new();
        let mut triangle_count = 0;
        for (builder, kind) in builders.into_iter().zip(CityGroundKind::ALL) {
            for (tile, mesh) in builder.build() {
                let mask = masks
                    .entry(tile)
                    .or_insert_with(|| traffic::TrafficMask::bake(&network, tile, images));
                triangle_count += mesh_triangle_count(&mesh);
                let mut entity = commands.spawn((
                    Name::new(format!("City ground {kind:?}")),
                    VistaTerrain(0),
                    NotShadowCaster,
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(self.materials.add(kind.material(
                        environment.weather,
                        &self.textures,
                        mask,
                    ))),
                    Transform::default(),
                ));
                if kind.is_yard() {
                    entity.insert(CityYardPresentation);
                } else {
                    entity.insert(CityStreetPresentation);
                }
            }
        }
        info!(traffic_tiles = masks.len(), wheel_segments = network.stroke_count(), triangles = triangle_count, scene = %environment.scene_digest, "Clipped city ground to canonical terrain triangles");
    }
}
