//! Bind supported camera eyes to the actual admitted natural terrain owner.

use super::*;
use crate::presentation::{ActiveVistaSurface, VistaTerrainMesh};
use adventuresim_tactical_core::{
    prelude::{SceneEnvironment, SceneTerrain},
    scene_input::VistaLevelIndex,
};
use bevy::ecs::system::SystemParam;
use std::num::NonZeroU64;

const SUPPORT_EYE: Metres = Metres::from_metres(super::super::super::STANDING_EYE_HEIGHT_METRES);

pub(super) struct Reader<'a> {
    terrain: &'a SceneTerrain,
    environment: &'a SceneEnvironment,
    surface: &'a ActiveVistaSurface,
}

pub(in super::super) struct SupportedCameras {
    pub(in super::super) cameras: catalog::Cameras,
    pub(in super::super) support: HeldSupport,
}

#[derive(Resource)]
pub(in super::super) struct HeldSupport {
    scene: Entity,
    environment: SceneEnvironment,
    revision: VistaRevision,
    chunks: Vec<SupportChunk>,
    samples: Vec<SupportSample>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
struct VistaRevision(NonZeroU64);

#[derive(Clone, Copy, PartialEq, Eq)]
struct SupportChunk {
    entity: Entity,
    level: VistaLevelIndex,
}

#[derive(Serialize)]
struct SupportSample {
    eye: Vec3,
    ground_height: Metres,
}

#[derive(SystemParam)]
pub(in super::super) struct Inspector<'w, 's> {
    held: Option<Res<'w, HeldSupport>>,
    surface: Option<Res<'w, ActiveVistaSurface>>,
    scenes: Query<'w, 's, (Entity, &'static SceneTerrain, &'static SceneEnvironment)>,
    chunks: Query<'w, 's, (Entity, &'static VistaTerrainMesh), With<Mesh3d>>,
}

impl Reader<'_> {
    fn height(&self, eye: Vec3) -> Result<Metres> {
        let height = self
            .surface
            .presented_height_at(&self.environment.scene_digest, self.terrain, eye.xz())
            .context("grown camera outside admitted presented terrain support")?;
        ensure!(height.is_finite(), "nonfinite grown camera ground");
        Ok(Metres::from_metres(height))
    }

    pub(super) fn clamp_eye(&self, mut eye: Vec3) -> Result<Vec3> {
        eye.y = eye.y.max(self.height(eye)?.metres() + SUPPORT_EYE.metres());
        ensure!(eye.is_finite(), "nonfinite supported camera eye");
        Ok(eye)
    }
}

impl HeldSupport {
    pub(in super::super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"scene_entity_bits": self.scene.to_bits(), "environment": self.environment, "vista_revision": self.revision, "chunks": self.chunks.iter().map(|chunk| serde_json::json!({"entity_bits":chunk.entity.to_bits(), "level":chunk.level})).collect::<Vec<_>>(), "samples":self.samples, "limits":"matching actual scene/environment, retained natural surface revision, required ring presence and final eye samples; not a byte seal of terrain meshes or material/GPU readiness"})
    }
}

impl Inspector<'_, '_> {
    pub(in super::super) fn check(&self, scene_digest: &str) -> Result<()> {
        let held = self
            .held
            .as_deref()
            .context("grown held camera support missing")?;
        let surface = self
            .surface
            .as_deref()
            .context("grown active vista support missing")?;
        let (scene, terrain, environment) = self.scenes.single()?;
        ensure!(
            scene == held.scene
                && environment == &held.environment
                && scene_digest == environment.scene_digest,
            "grown camera support scene/environment drift"
        );
        ensure!(
            surface.is_for_scene(&environment.scene_digest)
                && surface.revision() == held.revision.0.get()
                && !surface.is_urban_scene(&environment.scene_digest),
            "grown camera support revision drift"
        );
        let chunks = collect_chunks(self.chunks.iter());
        ensure!(
            chunks == held.chunks,
            "grown camera support chunk inventory drift"
        );
        let reader = Reader {
            terrain,
            environment,
            surface,
        };
        for sample in &held.samples {
            ensure!(
                reader.height(sample.eye)? == sample.ground_height,
                "grown camera support sample drift"
            );
        }
        Ok(())
    }
}

pub(in super::super) fn resolve(
    world: &mut World,
    geometry: &GrownGeometry,
    placements: &layout::RootPlacements,
) -> Result<SupportedCameras> {
    let mut scenes = world.query::<(Entity, &SceneTerrain, &SceneEnvironment)>();
    let mut chunks_query = world.query_filtered::<(Entity, &VistaTerrainMesh), With<Mesh3d>>();
    let (scene, terrain, environment) = scenes.single(world)?;
    ensure!(
        world
            .resource::<super::super::super::capture_state::SceneCaptureState>()
            .digest
            == environment.scene_digest,
        "grown camera scene digest mismatch"
    );
    let surface = world.resource::<ActiveVistaSurface>();
    ensure!(
        surface.is_for_scene(&environment.scene_digest)
            && !surface.is_urban_scene(&environment.scene_digest),
        "grown camera requires matching natural presented support"
    );
    let revision = VistaRevision(
        NonZeroU64::new(surface.revision()).context("grown camera vista support not admitted")?,
    );
    let chunks = collect_chunks(chunks_query.iter(world));
    let required = surface.retained_levels().collect::<Vec<_>>();
    ensure!(
        !required.is_empty()
            && required
                .iter()
                .all(|level| chunks.iter().any(|chunk| chunk.level == *level)),
        "grown camera required retained vista ring not presented"
    );
    let reader = Reader {
        terrain,
        environment,
        surface,
    };
    let library = geometry.bank().library();
    let bounds = library
        .models
        .get(geometry.variant().index())
        .context("grown camera source model missing")?
        .bounds;
    let cameras = catalog::Cameras::from_source(bounds, placements, &library.detail, &reader)?;
    let samples = cameras
        .eyes()
        .map(|eye| {
            Ok(SupportSample {
                eye,
                ground_height: reader.height(eye)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let held = HeldSupport {
        scene,
        environment: environment.clone(),
        revision,
        chunks,
        samples,
    };
    Ok(SupportedCameras {
        cameras,
        support: held,
    })
}

fn collect_chunks<'a>(
    chunks: impl Iterator<Item = (Entity, &'a VistaTerrainMesh)>,
) -> Vec<SupportChunk> {
    let mut chunks = chunks
        .map(|(entity, mesh)| SupportChunk {
            entity,
            level: mesh.0,
        })
        .collect::<Vec<_>>();
    chunks.sort_by_key(|chunk| chunk.entity.to_bits());
    chunks
}
