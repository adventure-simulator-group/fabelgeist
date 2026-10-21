//! Admit every original physical root before catalogue or child mutation.

use super::*;
use crate::presentation::obstacles::tree::presentation::PendingTreePresentation;
use adventuresim_tactical_core::prelude::{
    Collider, CollisionLayers, LayerMask, RigidBody, SceneObstacle, SceneObstacleOf, SceneTerrain,
    TACTICAL_TERRAIN_LAYER,
};

pub(super) struct PhysicalTreeRoot {
    pub entity: Entity,
    pub frame: Transform,
}

pub(super) fn admit(world: &mut World, scene: Entity) -> Result<Vec<PhysicalTreeRoot>> {
    let scene_count = world
        .query_filtered::<Entity, (With<SceneEnvironment>, With<SceneTerrain>)>()
        .iter(world)
        .count();
    ensure!(
        scene_count == 1,
        "grown generated roots require exactly one playable scene"
    );
    let entities = world
        .query_filtered::<Entity, With<PendingTreePresentation>>()
        .iter(world)
        .collect::<Vec<_>>();
    let terrain = world
        .get::<SceneTerrain>(scene)
        .context("grown root terrain")?;
    let mut roots = Vec::with_capacity(entities.len());
    for entity in entities {
        if let Some(root) = admit_root(world, scene, terrain, entity)? {
            roots.push(root);
        }
    }
    Ok(roots)
}

fn admit_root(
    world: &World,
    scene: Entity,
    terrain: &SceneTerrain,
    entity: Entity,
) -> Result<Option<PhysicalTreeRoot>> {
    let owner = world.get::<SceneObstacleOf>(entity);
    let obstacle = world.get::<SceneObstacle>(entity);
    let body = world.get::<RigidBody>(entity);
    let collider = world.get::<Collider>(entity);
    let layers = world.get::<CollisionLayers>(entity);
    let frame = world.get::<Transform>(entity);
    // Replicon commits the initial server entity before ordinary Update. A
    // collider's intermediate required defaults are not authoritative receipts.
    // Missing fields stay pending; present invalid fields remain hard errors.
    ensure!(
        owner.is_none_or(|owner| owner.0 == scene)
            && obstacle.is_none_or(|obstacle| *obstacle == SceneObstacle::Tree),
        "grown pending root does not belong to the actual generated tree scene"
    );
    ensure!(
        world.get::<ChildOf>(entity).is_none(),
        "grown physical root must remain unparented"
    );
    ensure!(
        body.is_none_or(|body| *body == RigidBody::Static)
            && layers
                .is_none_or(|layers| *layers
                    == CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL)),
        "grown physical root collider/body/layers changed"
    );
    if let Some(frame) = frame {
        validate_frame(terrain, frame)?;
    }
    let Some(frame) = frame else {
        return Ok(None);
    };
    if owner.is_none()
        || obstacle.is_none()
        || body.is_none()
        || collider.is_none()
        || layers.is_none()
    {
        return Ok(None);
    }
    Ok(Some(PhysicalTreeRoot {
        entity,
        frame: *frame,
    }))
}

fn validate_frame(terrain: &SceneTerrain, frame: &Transform) -> Result<()> {
    ensure!(
        frame.rotation == Quat::IDENTITY && frame.scale == Vec3::ONE,
        "grown physical root requires an identity rotation/scale"
    );
    let height = terrain
        .height_at(frame.translation.xz())
        .context("grown root is outside actual terrain")?;
    ensure!(
        height.is_finite()
            && frame.translation.y
                == height + super::super::super::source::source_normalization().y,
        "grown physical root is not centred above its actual ground"
    );
    Ok(())
}
