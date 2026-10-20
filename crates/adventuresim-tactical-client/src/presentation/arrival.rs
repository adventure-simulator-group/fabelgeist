//! Observe ordinary foreground arrival through one owned completed image.
//!
//! CPU producers and screenshot callbacks are separate evidence. This does not
//! certify exact screenshot-frame GPU instance multiplicity or performance.
use super::{ActiveTacticalScene, capture_image::CapturedPng};
use anyhow::{Context, Result, ensure};
use bevy::{
    prelude::*,
    render::view::screenshot::{Captured, Screenshot, ScreenshotCaptured},
};
use bridge::{Request, RequestId};
use serde_json::json;

mod bridge;
mod scene;
mod trees;

#[derive(Resource, Default)]
struct CaptureState {
    request: Option<RequestId>,
    phase: Phase,
}

struct PendingReadback {
    entity: Entity,
    request: Request,
    prerequisite: scene::Snapshot,
}

struct CapturedOwner {
    endpoint: Entity,
    scene: Entity,
    camera: Entity,
}

struct ReadbackEvent {
    request: RequestId,
    entity: Entity,
    is_captured: bool,
    image: Image,
}

#[derive(Default)]
enum Phase {
    #[default]
    Idle,
    Waiting,
    Pending(PendingReadback),
    Complete(CapturedOwner),
    Failed,
}

impl CaptureState {
    fn advance(&mut self, world: &mut World, request: Request) -> Result<()> {
        if self.request != Some(request.id) {
            self.request = Some(request.id);
            self.phase = Phase::Waiting;
        }
        match &self.phase {
            Phase::Waiting => {
                let Some(prerequisite) = scene::inspect(world, &request.target)? else {
                    return Ok(());
                };
                let id = request.id;
                let entity = world
                    .spawn(Screenshot::primary_window())
                    .observe(
                        move |event: On<ScreenshotCaptured>,
                              receipts: Query<(), With<Captured>>,
                              mut commands: Commands| {
                            let captured = ReadbackEvent {
                                request: id,
                                entity: event.entity,
                                is_captured: receipts.contains(event.entity),
                                image: event.image.clone(),
                            };
                            commands.queue(move |world: &mut World| finish(world, captured));
                        },
                    )
                    .id();
                bridge::pending(id, prerequisite.report())?;
                self.phase = Phase::Pending(PendingReadback {
                    entity,
                    request,
                    prerequisite,
                });
            }
            Phase::Complete(owner) => {
                ensure!(
                    world
                        .get::<adventuresim_tactical_netcode::aeronet::io::Session>(owner.endpoint)
                        .is_some()
                        && world.resource::<ActiveTacticalScene>().entity == Some(owner.scene)
                        && world
                            .get::<super::TacticalGameplayCamera>(owner.camera)
                            .is_some(),
                    "captured arrival owner is no longer active"
                );
            }
            Phase::Idle | Phase::Pending(_) | Phase::Failed => {}
        }
        Ok(())
    }

    fn captured(&mut self, world: &mut World, event: ReadbackEvent) -> Result<()> {
        ensure!(
            self.request == Some(event.request),
            "stale arrival readback state"
        );
        let Phase::Pending(pending) = std::mem::replace(&mut self.phase, Phase::Failed) else {
            anyhow::bail!("unsolicited or duplicate arrival readback");
        };
        ensure!(
            event.entity == pending.entity && event.is_captured,
            "arrival readback entity/receipt mismatch"
        );
        let current = scene::inspect(world, &pending.request.target)?
            .context("arrival producers incomplete at readback")?;
        ensure!(
            current.same_owner(&pending.prerequisite)
                && current.frame.0 >= pending.prerequisite.frame.0,
            "arrival readback owner or producer census changed"
        );
        let image = CapturedPng::encode(event.image)?;
        ensure!(
            UVec2::new(image.width, image.height) == current.extent,
            "arrival readback extent differs from target"
        );
        let receipt = json!({"prerequisite":pending.prerequisite.report(),"callback":current.report(),
            "screenshot_entity":event.entity.to_bits().to_string(),
            "scope":"current-owner completed original image and CPU producer census; exact GPU screenshot frame, per-patch GPU multiplicity, source bytes and physical memory are not certified; host receipt time defines the measured upper bound"});
        bridge::complete(event.request, receipt, image)?;
        self.phase = Phase::Complete(CapturedOwner {
            endpoint: current.endpoint,
            scene: current.scene,
            camera: current.camera,
        });
        Ok(())
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<CaptureState>()
        .add_systems(Last, update);
}

fn update(world: &mut World) -> bevy::prelude::Result {
    if !bridge::enabled() {
        return Ok(());
    }
    let (observation, empty) = scene::observation(world);
    bridge::observe(observation, empty)?;
    let Some(request) = bridge::request()? else {
        return Ok(());
    };
    let id = request.id;
    world.resource_scope(|world, mut state: Mut<CaptureState>| {
        if let Err(error) = state.advance(world, request) {
            state.phase = Phase::Failed;
            bridge::fail(id, error);
        }
    });
    Ok(())
}

fn finish(world: &mut World, event: ReadbackEvent) {
    if !bridge::is_current(event.request) {
        return;
    }
    let id = event.request;
    world.resource_scope(|world, mut state: Mut<CaptureState>| {
        if let Err(error) = state.captured(world, event) {
            bridge::fail(id, error);
        }
    });
}
