//! Shared, terrain-conforming city ground presentation at every viewing distance.

use super::*;

mod activity;
mod material;
mod mesh;
mod support;
pub(in crate::presentation::vista) use support::GroundSupport;

use material::CityGroundKind;
pub(in crate::presentation) use material::CityGroundMaterial;
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

#[expect(
    clippy::too_many_arguments,
    reason = "city ground consumes shared city geometry, terrain, weather, and asset stores"
)]
pub(super) fn spawn(
    commands: &mut Commands,
    streets: &[CityStreetPatch],
    yards: &[CityYardPatch],
    groups: &[FurnitureGroup],
    support: &GroundSupport,
    environment: &SceneEnvironment,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<CityGroundMaterial>,
    textures: &ProceduralTextureAssets,
) {
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
    let mut triangle_count = 0;
    for (builder, kind) in builders.into_iter().zip(CityGroundKind::ALL) {
        let Some(mesh) = builder.build() else {
            continue;
        };
        triangle_count += mesh_triangle_count(&mesh);
        let mut entity = commands.spawn((
            Name::new(format!("City ground {kind:?}")),
            VistaTerrain(0),
            NotShadowCaster,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(kind.material(environment.weather, textures))),
            Transform::default(),
        ));
        if kind.is_yard() {
            entity.insert(CityYardPresentation);
        } else {
            entity.insert(CityStreetPresentation);
        }
    }
    info!(triangles = triangle_count, scene = %environment.scene_digest, "Clipped city ground to canonical terrain triangles");
}
