//! Current connection, producer completion and the actual normal gameplay view.
use super::super::{
    ActiveTacticalScene, GrownPartBinding, ProceduralRockVisual, SceneEnvironment,
    TacticalGameplayCamera, TacticalGraphicsSettings, TreeContentRequest,
    ground_scatter::{
        AcceptedLitterWork, GroundScatterPresented, instanced_grass::InstancedGrassPresented,
        plants::PlantsPresented,
    },
    obstacles::tree::{GrownCatalogueAssets, TreePresentationCache},
    terrain::{PendingTerrainPresentation, ScenePresentationOf},
    vista::{ActiveVistaSurface, GrownVistaPart, VistaTerrainMesh, VistaTreeCensus},
};
use super::{bridge::Target, trees};
use adventuresim_environment::render::Level;
use adventuresim_tactical_core::prelude::{
    SceneId, SceneObstacle, SceneObstacleCensus, SceneObstacleOf, SceneTreeContent,
};
use adventuresim_tactical_netcode::{aeronet::io::Session, prelude::AdventureSimulatorClient};
use anyhow::{Context, Result, ensure};
use bevy::{
    camera::{Projection, visibility::VisibleEntities},
    diagnostic::FrameCount,
    prelude::*,
};
use serde_json::{Value, json};
use std::{any::TypeId, sync::Arc};

pub(super) struct Snapshot {
    pub endpoint: Entity,
    pub scene: Entity,
    pub camera: Entity,
    pub extent: UVec2,
    pub frame: FrameCount,
    pub catalogue: Arc<GrownCatalogueAssets>,
    receipt: Value,
}

struct Population {
    trees: usize,
    rocks: usize,
    selected_parts: usize,
    full: usize,
    middle: usize,
    coarse: usize,
}

impl Snapshot {
    fn from_producers(
        world: &mut World,
        target: &Target,
        endpoint: Entity,
        scene: Entity,
        catalogue: Arc<GrownCatalogueAssets>,
        population: Population,
        vista_parts: usize,
    ) -> Result<Option<Self>> {
        let cameras = world
            .query_filtered::<Entity, With<TacticalGameplayCamera>>()
            .iter(world)
            .collect::<Vec<_>>();
        if cameras.len() != 1 {
            return Ok(None);
        }
        let camera_entity = cameras[0];
        let camera = world
            .get::<Camera>(camera_entity)
            .context("arrival gameplay camera missing")?;
        if !camera.is_active
            || camera.physical_target_size() != Some(target.extent)
            || camera.physical_viewport_size() != Some(target.extent)
        {
            return Ok(None);
        }
        let Some(lighting) =
            super::super::atmosphere::receipt::completed_probe(world, scene, camera_entity)
        else {
            return Ok(None);
        };
        let visible = world
            .get::<VisibleEntities>(camera_entity)
            .context("arrival native visible view missing")?
            .get(TypeId::of::<Mesh3d>())
            .to_vec();
        ensure!(!visible.is_empty(), "arrival gameplay view is empty");
        let visible_local = visible
            .iter()
            .filter(|entity| {
                world
                    .get::<GrownPartBinding>(**entity)
                    .is_some_and(|part| part.scene == scene)
            })
            .count();
        let visible_vista = visible
            .iter()
            .filter(|entity| {
                world
                    .get::<GrownVistaPart>(**entity)
                    .is_some_and(|part| part.0.scene == scene)
            })
            .count();
        let terrain = world
            .query::<(&ScenePresentationOf, Option<&Mesh3d>)>()
            .iter(world)
            .filter(|(owner, mesh)| owner.0 == scene && mesh.is_some())
            .count();
        if terrain == 0 {
            return Ok(None);
        }
        let vista_terrain = world.query::<&VistaTerrainMesh>().iter(world).count();
        let census = world
            .get::<VistaTreeCensus>(scene)
            .context("arrival vista census missing")?;
        let work = world
            .get::<AcceptedLitterWork>(scene)
            .context("arrival litter receipt missing")?;
        let receipt = json!({"scene_id":target.scene.0,"scene_digest":target.digest.to_hex().as_str(),
        "endpoint_entity":endpoint.to_bits().to_string(),"scene_entity":scene.to_bits().to_string(),
        "camera_entity":camera_entity.to_bits().to_string(),"extent":target.extent.to_array(),
        "lighting":lighting,
        "main_frame":world.resource::<FrameCount>().0,"bank":catalogue.bank_assets().bank().id(),
        "sources":catalogue.sources().iter().map(|source|json!({"plant":source.plant.word().to_string(),"variant":source.variant.index()})).collect::<Vec<_>>(),
        "catalogue_logical_mesh_bytes":catalogue.logical_mesh_bytes().bytes(),
        "producers":{"physical_trees":population.trees,"physical_rocks":population.rocks,
            "vista_roots":census.roots.len(),"vista_parts":vista_parts,"terrain_meshes":terrain,
            "vista_terrain_meshes":vista_terrain,"litter":work,"grass_enabled":world.resource::<TacticalGraphicsSettings>().config.grass.enabled,
            "grass_density_scale":world.resource::<TacticalGraphicsSettings>().config.grass.density_scale,
            "maximum_vista_lods":world.resource::<TacticalGraphicsSettings>().config.rendering.vista.maximum_lods},
        "selected":{"full":population.full,"middle":population.middle,"coarse":population.coarse,"parts":population.selected_parts},
        "view":{"native_visible_meshes":visible.len(),"local_tree_parts":visible_local,"vista_tree_parts":visible_vista,
            "world_from_camera":world.get::<GlobalTransform>(camera_entity).map(|frame|frame.to_matrix().to_cols_array()),
            "projection":world.get::<Projection>(camera_entity).map(|projection|format!("{projection:?}"))},
        "scope":"current connection bank admission and complete CPU producers; no per-patch GPU multiplicity or exact screenshot-frame association"});
        Ok(Some(Snapshot {
            endpoint,
            scene,
            camera: camera_entity,
            extent: target.extent,
            frame: *world.resource::<FrameCount>(),
            catalogue,
            receipt,
        }))
    }

    pub(super) fn report(&self) -> Value {
        self.receipt.clone()
    }

    pub(super) fn same_owner(&self, previous: &Self) -> bool {
        self.endpoint == previous.endpoint
            && self.scene == previous.scene
            && self.camera == previous.camera
            && self.extent == previous.extent
            && Arc::ptr_eq(&self.catalogue, &previous.catalogue)
            && self.receipt["producers"] == previous.receipt["producers"]
            && self.receipt["lighting"] == previous.receipt["lighting"]
    }
}

pub(super) fn observation(world: &mut World) -> (Value, bool) {
    let endpoints = world.query::<(Entity,&AdventureSimulatorClient,Has<Session>)>()
        .iter(world).map(|(entity,client,session)|json!({"entity":entity.to_bits().to_string(),
            "server_url":client.server_url,"character_id":client.player_id.to_string(),"session":session})).collect::<Vec<_>>();
    let empty = endpoints.is_empty();
    let scene = world
        .get_resource::<ActiveTacticalScene>()
        .and_then(|active| active.entity);
    let current = scene.map(|scene| json!({"entity":scene.to_bits().to_string(),
        "scene_id":world.get::<SceneId>(scene).map(|id|id.0.as_str()),
        "scene_digest":world.get::<SceneEnvironment>(scene).map(|environment|environment.scene_digest.as_str())}));
    (
        json!({"main_frame":world.get_resource::<FrameCount>().map(|frame|frame.0),
        "endpoints":endpoints,"active_scene":current}),
        empty,
    )
}

pub(super) fn inspect(world: &mut World, target: &Target) -> Result<Option<Snapshot>> {
    let endpoints = world
        .query_filtered::<(Entity, &AdventureSimulatorClient), With<Session>>()
        .iter(world)
        .filter(|(_, client)| {
            client.player_id == target.character.0
                && adventuresim_tactical_netcode::client::normalize_server_url(&client.server_url)
                    == target.server_url
        })
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    if endpoints.len() != 1 {
        return Ok(None);
    }
    let endpoint = endpoints[0];
    let Some(scene) = world.resource::<ActiveTacticalScene>().entity else {
        return Ok(None);
    };
    let Some(id) = world.get::<SceneId>(scene) else {
        return Ok(None);
    };
    let Some(environment) = world.get::<SceneEnvironment>(scene).cloned() else {
        return Ok(None);
    };
    if *id != target.scene || environment.scene_digest != target.digest.to_hex().as_str() {
        return Ok(None);
    }
    let Some(request) = world.get::<TreeContentRequest>(scene).cloned() else {
        return Ok(None);
    };
    ensure!(
        request.family() == SceneTreeContent::Grown
            && world.get::<SceneTreeContent>(scene) == Some(&SceneTreeContent::Grown),
        "arrival target did not receive grown tree intent"
    );
    if !request.has_current_admission(world, endpoint, scene) {
        return Ok(None);
    }
    if !producers_complete(world, scene) {
        return Ok(None);
    }
    let catalogue = {
        let cache = world.resource::<TreePresentationCache>();
        if !cache.ready_for_vista(scene, &environment, &request)? {
            return Ok(None);
        }
        cache.lease_grown_catalogue(scene, &environment)?
    };
    let Some(population) = population(world, scene, &catalogue)? else {
        return Ok(None);
    };
    let Some(vista_parts) = trees::vista(world, scene, &catalogue)? else {
        return Ok(None);
    };
    Snapshot::from_producers(
        world,
        target,
        endpoint,
        scene,
        catalogue,
        population,
        vista_parts,
    )
}

fn producers_complete(world: &World, scene: Entity) -> bool {
    world.get::<PendingTerrainPresentation>(scene).is_none()
        && world.get::<InstancedGrassPresented>(scene).is_some()
        && world.get::<GroundScatterPresented>(scene).is_some()
        && world.get::<PlantsPresented>(scene).is_some()
        && world.get::<AcceptedLitterWork>(scene).is_some()
        && world.get::<VistaTreeCensus>(scene).is_some()
        && world
            .resource::<TacticalGraphicsSettings>()
            .config
            .grass
            .enabled
        && world
            .resource::<TacticalGraphicsSettings>()
            .config
            .grass
            .density_scale
            > 0.0
        && world
            .get::<SceneEnvironment>(scene)
            .is_some_and(|environment| {
                world
                    .resource::<ActiveVistaSurface>()
                    .is_for_scene(&environment.scene_digest)
            })
}

fn population(
    world: &mut World,
    scene: Entity,
    catalogue: &GrownCatalogueAssets,
) -> Result<Option<Population>> {
    let Some(expected) = world.get::<SceneObstacleCensus>(scene).copied() else {
        return Ok(None);
    };
    let roots = world
        .query::<(Entity, &SceneObstacleOf, &SceneObstacle)>()
        .iter(world)
        .filter(|(_, owner, _)| owner.0 == scene)
        .map(|(entity, _, kind)| (entity, *kind))
        .collect::<Vec<_>>();
    let mut actual = Population {
        trees: 0,
        rocks: 0,
        selected_parts: 0,
        full: 0,
        middle: 0,
        coarse: 0,
    };
    for (entity, kind) in roots {
        match kind {
            SceneObstacle::Tree => {
                actual.trees += 1;
                let Some(installed) = trees::local(world, scene, entity, catalogue)? else {
                    return Ok(None);
                };
                actual.selected_parts += installed.parts;
                match installed.level {
                    Level::Full => actual.full += 1,
                    Level::Middle => actual.middle += 1,
                    Level::Coarse => actual.coarse += 1,
                }
            }
            SceneObstacle::Rock(_) => {
                actual.rocks += 1;
                if world.get::<ProceduralRockVisual>(entity).is_none()
                    || world.get::<Mesh3d>(entity).is_none()
                {
                    return Ok(None);
                }
            }
        }
    }
    if actual.trees != usize::try_from(expected.trees().roots())?
        || actual.rocks != usize::try_from(expected.rocks().roots())?
    {
        return Ok(None);
    }
    Ok(Some(actual))
}
