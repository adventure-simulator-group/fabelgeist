//! Request/view-owned readbacks; PNG encoding follows the receipt timestamp.

use super::{
    bridge,
    session::{Arrival, Session},
};
use adventuresim_environment::render::{
    ForestGeometry,
    benchmark::{ForestFrameReceipt, ForestRequestId},
};
use anyhow::{Result, ensure};
use bevy::{
    prelude::*,
    render::view::screenshot::{Captured, Screenshot, ScreenshotCaptured},
};
use serde::Serialize;

pub(super) struct PendingCapture {
    entity: Entity,
    camera: Entity,
    role: CaptureRole,
    receipt: ForestFrameReceipt,
}

#[derive(Serialize)]
pub(super) struct Readback {
    role: CaptureRole,
    prerequisite_completed_frame: ForestFrameReceipt,
    elapsed_from_request_millis: f64,
    width: u32,
    height: u32,
    png_bytes: usize,
    png_blake3: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CaptureRole {
    Early,
    SettledFirst,
    SettledSecond,
}

impl PendingCapture {
    pub fn evidence(&self) -> serde_json::Value {
        serde_json::json!({"entity_bits":self.entity.to_bits(),
            "camera_entity_bits":self.camera.to_bits(), "role":self.role,
            "prerequisite_completed_frame":self.receipt})
    }
}

pub(super) fn request(
    world: &mut World,
    arrival: &mut Arrival,
    camera: Entity,
    role: CaptureRole,
) -> Result<()> {
    if arrival.pending.is_some() {
        return Ok(());
    }
    let probe = arrival
        .probe
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("missing readback probe"))?;
    let receipt = probe
        .latest_completed_frame()
        .ok_or_else(|| anyhow::anyhow!("missing completed frame"))?;
    ensure!(
        receipt.request() == arrival.request && receipt.camera() == camera,
        "readback receipt ownership mismatch"
    );
    let request = arrival.request;
    let entity = world
        .spawn(Screenshot::primary_window())
        .observe(
            move |event: On<ScreenshotCaptured>,
                  mut session: ResMut<Session>,
                  receipts: Query<(), With<Captured>>| {
                if let Err(error) = captured(
                    &mut session,
                    request,
                    &event,
                    receipts.contains(event.entity),
                ) {
                    session.fail(error.to_string());
                }
            },
        )
        .id();
    arrival.pending = Some(PendingCapture {
        entity,
        camera,
        role,
        receipt,
    });
    Ok(())
}

pub(super) fn validate_inventory(world: &mut World, arrival: &Arrival) -> Result<()> {
    let probe = arrival
        .probe
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("missing final probe"))?;
    let native = probe
        .native_residency()
        .ok_or_else(|| anyhow::anyhow!("missing final inventory"))?;
    let mut entities = world.query_filtered::<Entity, With<ForestGeometry>>();
    ensure!(
        native.matches_scene_entities(entities.iter(world)),
        "stale/missing review geometry"
    );
    let mut screenshots = world.query_filtered::<Entity, With<Screenshot>>();
    ensure!(
        screenshots.iter(world).next().is_none(),
        "undrained screenshot remains"
    );
    Ok(())
}

fn captured(
    session: &mut Session,
    request: ForestRequestId,
    event: &ScreenshotCaptured,
    is_captured: bool,
) -> Result<()> {
    let arrival = session
        .arrival
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("missing readback arrival"))?;
    ensure!(arrival.request == request, "stale screenshot callback");
    let pending = arrival
        .pending
        .take()
        .ok_or_else(|| anyhow::anyhow!("unsolicited readback"))?;
    ensure!(
        event.entity == pending.entity && session.camera == Some(pending.camera),
        "readback entity/view mismatch"
    );
    ensure!(is_captured, "readback lacks Captured receipt");
    ensure!(
        !bridge::has_render_failure() && arrival.probe.as_ref().is_some_and(|p| !p.status().failed),
        "readback followed renderer failure"
    );
    let elapsed_from_request_millis = arrival.started.elapsed().as_secs_f64() * 1000.0;
    let image = crate::presentation::capture_image::CapturedPng::encode(event.image.clone())?;
    arrival.readbacks.push(Readback {
        role: pending.role,
        prerequisite_completed_frame: pending.receipt,
        elapsed_from_request_millis,
        width: image.width,
        height: image.height,
        png_bytes: image.bytes.len(),
        png_blake3: image.blake3,
    });
    bridge::push_image(image.bytes)?;
    Ok(())
}
