//! Shared, terrain-conforming city ground presentation at every viewing distance.

use super::*;

mod activity;
mod material;
mod mesh;
mod partition;
pub(in crate::presentation) mod prepared;
pub(crate) mod streaming;
mod support;
mod traffic;
pub(in crate::presentation) use support::GroundSupport;

use material::CityGroundKind;
pub(crate) use material::CityGroundMaterial;
use mesh::CitySurfaceMeshBuilder;

#[derive(bevy::ecs::system::SystemParam)]
pub(in crate::presentation) struct CityGroundAssets<'w> {
    pub(super) materials: ResMut<'w, Assets<CityGroundMaterial>>,
    pub(super) textures: Res<'w, ProceduralTextureAssets>,
    pub(super) streaming: Option<Res<'w, streaming::StreamCityTraffic>>,
}

pub(in crate::presentation) struct FocusedPavingPlacement {
    pub root: Entity,
    pub frame: crate::presentation::buildings::CityFrame,
    pub weather: WeatherSnapshot,
}

pub(super) struct UrbanGround(UrbanGroundLookup);

impl UrbanGround {
    pub(super) fn new(streets: &[CityStreetPatch], yards: &[CityYardPatch]) -> Self {
        Self(UrbanGroundLookup::new(streets, yards, &[]))
    }

    pub(super) fn suppresses_grass(&self, point: Vec2) -> bool {
        self.0.suppresses_grass(point)
    }
}

#[derive(Component)]
pub(crate) struct CityStreetPresentation;

#[derive(Component)]
pub(crate) struct CityYardPresentation;

impl CityGroundAssets<'_> {
    /// Install the complete worker-built paving as map-owned children. Traffic
    /// stays neutral until its shared bake can be scheduled independently; no
    /// actor traffic resource or scene bundle is installed by this path.
    pub(in crate::presentation) fn spawn_focused<'a>(
        &mut self,
        commands: &mut Commands,
        prepared: &'a prepared::PreparedCityGround,
        placement: FocusedPavingPlacement,
        meshes: &mut Assets<Mesh>,
        images: &mut Assets<Image>,
    ) -> Result<Vec<Handle<CityGroundMaterial>>, &'a super::geographic_ground::GeographicGroundError>
    {
        let owner = crate::presentation::ownership::PresentationOwner::RegionalMap;
        let uploaded = prepared.meshes(owner, meshes)?;
        let mask = traffic::TrafficMask::neutral(images);
        let mut materials = Vec::new();
        for (_, kind, mesh, _) in uploaded {
            let mut material = kind.material(placement.weather, &self.textures, &mask);
            material.extension.set_geographic_frame(placement.frame);
            let material = self.materials.add(material);
            materials.push(material.clone());
            let mut entity = commands.spawn((
                Name::new(format!("Map city ground {kind:?}")),
                owner,
                owner.render_layers(),
                NotShadowCaster,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                ChildOf(placement.root),
            ));
            if kind.is_yard() {
                entity.insert(CityYardPresentation);
            } else {
                entity.insert(CityStreetPresentation);
            }
        }
        Ok(materials)
    }

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
        let build_ground = || {
            std::sync::Arc::new(prepared::PreparedCityGround::new(
                &bundle.streets,
                &bundle.yards,
                &bundle.furniture_groups,
                support,
            ))
        };
        #[cfg(target_family = "wasm")]
        let prepared = match crate::presentation::generation::landscape::ground(
            crate::presentation::ownership::PresentationOwner::Scene,
            &environment.scene_digest,
        ) {
            Ok(prepared) => prepared.unwrap_or_else(build_ground),
            Err(error) => {
                warn!(%error, "Could not access prepared city ground");
                return;
            }
        };
        #[cfg(not(target_family = "wasm"))]
        let prepared = build_ground();
        let network = self
            .streaming
            .is_none()
            .then(|| traffic::TrafficNetwork::new(streets));
        let neutral = self
            .streaming
            .is_some()
            .then(|| traffic::TrafficMask::neutral(images));
        let mut masks = std::collections::BTreeMap::new();
        let mut triangle_count = 0;
        let uploaded = match prepared.meshes(
            crate::presentation::ownership::PresentationOwner::Scene,
            meshes,
        ) {
            Ok(uploaded) => uploaded,
            Err(error) => {
                warn!(%error, "Could not upload canonical city ground");
                return;
            }
        };
        for (tile, kind, mesh, triangles) in uploaded {
            let mask = masks.entry(*tile).or_insert_with(|| match &network {
                Some(network) => traffic::TrafficMask::bake(network, *tile, images),
                None => neutral.as_ref().expect("streaming neutral mask").clone(),
            });
            triangle_count += triangles;
            let mut entity = commands.spawn((
                Name::new(format!("City ground {kind:?}")),
                VistaTerrain(adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0)),
                NotShadowCaster,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(self.materials.add(kind.material(
                    environment.weather,
                    &self.textures,
                    mask,
                ))),
                Transform::default(),
            ));
            if self.streaming.is_some() {
                entity.insert(streaming::StreamedTrafficTile(*tile));
            }
            if kind.is_yard() {
                entity.insert(CityYardPresentation);
            } else {
                entity.insert(CityStreetPresentation);
            }
        }
        if let Some(neutral) = neutral {
            commands.insert_resource(streaming::CityTrafficResidency::new(
                streets.clone(),
                neutral,
                prepared.traffic.clone(),
            ));
        }
        info!(traffic_tiles = masks.len(), wheel_segments = network.as_ref().map_or(0, traffic::TrafficNetwork::stroke_count), triangles = triangle_count, scene = %environment.scene_digest, "Clipped city ground to canonical terrain triangles");
    }
}
