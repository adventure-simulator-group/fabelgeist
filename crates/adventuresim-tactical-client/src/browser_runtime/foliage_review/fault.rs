//! One-shot ownership faults confined to the opt-in diagnostic scene.

use super::{scene, session::Session, wire::MAX_DOCUMENT_BYTES};
use adventuresim_environment::render::{ForestGeometry, GroundPart};
use anyhow::{Result, ensure};
use bevy::prelude::*;
use bevy_eidolon::prepass::CullComputeCamera;
use serde::{Deserialize, Serialize};

/// Keep the terrain mesh alive while deleting its owner, so this fault tests
/// exact scene membership rather than merely waiting for a missing GPU mesh.
#[derive(Resource)]
struct RemovedGroundAssets {
    _mesh: Mesh3d,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Fault {
    InactiveCullView,
    AmbiguousCullView,
    MissingGround,
    HiddenGround,
}

#[derive(Clone, Copy, Default, Serialize)]
#[serde(tag = "state", content = "kind", rename_all = "snake_case")]
pub(super) enum Injection {
    #[default]
    Unarmed,
    Armed(Fault),
    Applied(Fault),
}

impl Fault {
    pub fn parse(json: &str) -> Result<Self> {
        ensure!(json.len() <= MAX_DOCUMENT_BYTES, "oversized fault document");
        Ok(serde_json::from_str(json)?)
    }

    fn apply(self, world: &mut World, camera: Entity) -> Result<()> {
        match self {
            Self::InactiveCullView | Self::AmbiguousCullView => {
                let transform = world
                    .get::<Transform>(camera)
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("fault requires the admitted camera pose"))?;
                let projection = world
                    .get::<Projection>(camera)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("fault requires the admitted camera lens"))?;
                // This extra diagnostic camera survives ordinary scene isolation.
                // It deliberately lacks ForestViewCamera, leaving the owned view
                // valid while challenging Eidolon's actual culling-source query.
                world.spawn((
                    scene::ReviewOwned,
                    scene::ReviewCamera,
                    Camera3d::default(),
                    CullComputeCamera,
                    Camera {
                        is_active: matches!(self, Self::AmbiguousCullView),
                        order: 1,
                        ..default()
                    },
                    transform,
                    projection,
                ));
            }
            Self::MissingGround | Self::HiddenGround => {
                let mut ground =
                    world.query_filtered::<(Entity, &GroundPart), With<ForestGeometry>>();
                let terrain: Vec<_> = ground
                    .iter(world)
                    .filter_map(|(entity, part)| {
                        matches!(part, GroundPart::Terrain).then_some(entity)
                    })
                    .collect();
                let [terrain] = terrain.as_slice() else {
                    anyhow::bail!("fault requires exactly one current diagnostic terrain");
                };
                if matches!(self, Self::MissingGround) {
                    let retained = RemovedGroundAssets {
                        _mesh: world
                            .get::<Mesh3d>(*terrain)
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("diagnostic terrain has no mesh"))?,
                    };
                    world.insert_resource(retained);
                    world.despawn(*terrain);
                } else {
                    *world
                        .get_mut::<Visibility>(*terrain)
                        .ok_or_else(|| anyhow::anyhow!("diagnostic terrain has no visibility"))? =
                        Visibility::Hidden;
                }
            }
        }
        Ok(())
    }
}

impl Injection {
    pub fn arm(&mut self, fault: Fault) -> Result<()> {
        ensure!(
            matches!(self, Self::Unarmed),
            "ownership fault already selected"
        );
        *self = Self::Armed(fault);
        Ok(())
    }

    fn take(&mut self) -> Option<Fault> {
        let Self::Armed(fault) = *self else {
            return None;
        };
        *self = Self::Applied(fault);
        Some(fault)
    }
}

pub(super) fn apply(world: &mut World) {
    world.resource_scope(|world, mut session: Mut<Session>| {
        let Some(fault) = session.arrival.as_mut().and_then(|a| a.injection.take()) else {
            return;
        };
        let result = session
            .camera
            .ok_or_else(|| anyhow::anyhow!("ownership fault has no admitted camera"))
            .and_then(|camera| fault.apply(world, camera));
        if let Err(error) = result {
            session.fail(error.to_string());
        }
    });
}
