//! Request identity is reserved before ordinary EnterTactical is queued.
use super::super::capture_image::CapturedPng;
use adventuresim_tactical_core::prelude::CharacterId;
use adventuresim_tactical_core::prelude::SceneId;
use anyhow::{Context, Result, ensure};
use bevy::prelude::UVec2;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex, OnceLock},
};
use wasm_bindgen::prelude::*;

const MAX_REQUEST_BYTES: usize = 4096;
const MAX_SCENE_ID_BYTES: usize = 128;
const MAX_CAPTURE_PIXELS: u64 = 4096 * 4096;
const MAX_FAILURE_CHARACTERS: usize = 4096;

static BRIDGE: OnceLock<Mutex<Bridge>> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
pub(super) struct RequestId(NonZeroU32);

#[derive(Clone)]
pub(super) struct Request {
    pub id: RequestId,
    pub target: Arc<Target>,
}

pub(super) struct Target {
    pub scene: SceneId,
    pub digest: blake3::Hash,
    /// Normalized external transport address, matching the actual client adapter.
    pub server_url: String,
    pub character: CharacterId,
    pub extent: UVec2,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireTarget {
    scene_id: String,
    scene_digest: String,
    server_addr: String,
    character_id: u64,
    width: u32,
    height: u32,
}

#[derive(Default)]
struct Bridge {
    last: u32,
    active: Option<Request>,
    observed: Option<Value>,
    endpoints_empty: bool,
    state: Value,
    image: Option<CapturedPng>,
    renderer_failure: Option<String>,
}

impl Target {
    fn parse(input: &str) -> Result<Self> {
        ensure!(
            input.len() <= MAX_REQUEST_BYTES,
            "arrival request exceeds bound"
        );
        let wire: WireTarget = serde_json::from_str(input)?;
        ensure!(
            !wire.scene_id.is_empty() && wire.scene_id.len() <= MAX_SCENE_ID_BYTES,
            "invalid arrival scene identity"
        );
        ensure!(!wire.server_addr.is_empty(), "arrival endpoint is empty");
        let digest = blake3::Hash::from_hex(&wire.scene_digest)?;
        ensure!(
            wire.scene_digest == digest.to_hex().as_str(),
            "arrival digest must be canonical lowercase hex"
        );
        ensure!(
            wire.width > 0
                && wire.height > 0
                && u64::from(wire.width) * u64::from(wire.height) <= MAX_CAPTURE_PIXELS,
            "arrival capture extent exceeds bound"
        );
        Ok(Self {
            scene: SceneId(wire.scene_id),
            digest,
            server_url: adventuresim_tactical_netcode::client::normalize_server_url(
                &wire.server_addr,
            ),
            character: CharacterId(wire.character_id),
            extent: UVec2::new(wire.width, wire.height),
        })
    }

    pub(super) fn report(&self) -> Value {
        json!({"scene_id":self.scene.0,"scene_digest":self.digest.to_hex().as_str(),
            "server_url":self.server_url,"character_id":self.character.0.to_string(),
            "extent":self.extent.to_array()})
    }
}

impl RequestId {
    fn word(self) -> u32 {
        self.0.get()
    }
}

#[wasm_bindgen]
pub fn wasm_tactical_arrival_begin(input: String) -> Result<u32, JsValue> {
    let outcome = (|| -> Result<u32> {
        let target = Arc::new(Target::parse(&input)?);
        let mut bridge = locked()?;
        ensure!(
            bridge.observed.is_some() && bridge.endpoints_empty,
            "arrival begin requires an actual empty-endpoint frame acknowledgement"
        );
        ensure!(
            bridge.renderer_failure.is_none(),
            "renderer has a sticky failure"
        );
        let next = bridge
            .last
            .checked_add(1)
            .and_then(NonZeroU32::new)
            .context("arrival request counter exhausted")?;
        let id = RequestId(next);
        bridge.last = id.word();
        bridge.active = Some(Request { id, target });
        bridge.image = None;
        bridge.state = json!({"state":"waiting","request":id});
        Ok(id.word())
    })();
    outcome.map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn wasm_tactical_arrival_status() -> String {
    locked()
        .map(|bridge| {
            json!({"observation":bridge.observed,"capture":bridge.state,
        "target":bridge.active.as_ref().map(|request|request.target.report()),
        "renderer_failure":bridge.renderer_failure})
            .to_string()
        })
        .unwrap_or_else(|error| json!({"error":error.to_string()}).to_string())
}

#[wasm_bindgen]
pub fn wasm_tactical_arrival_png(request: u32) -> Result<Vec<u8>, JsValue> {
    let outcome = (|| -> Result<Vec<u8>> {
        let bridge = locked()?;
        ensure!(
            bridge
                .active
                .as_ref()
                .is_some_and(|active| active.id.word() == request),
            "stale arrival image request"
        );
        Ok(bridge
            .image
            .as_ref()
            .context("arrival image is not captured")?
            .bytes
            .clone())
    })();
    outcome.map_err(|error| JsValue::from_str(&error.to_string()))
}

/// The harness reports actual device-loss and uncaptured WebGPU errors here.
#[wasm_bindgen]
pub fn wasm_tactical_arrival_device_failed(message: String) {
    if let Ok(mut bridge) = locked() {
        bridge
            .renderer_failure
            .get_or_insert_with(|| message.chars().take(MAX_FAILURE_CHARACTERS).collect());
        bridge.state = json!({"state":"failed","error":bridge.renderer_failure});
        bridge.image = None;
    }
}

pub(super) fn enabled() -> bool {
    BRIDGE.get().is_some()
}

pub(super) fn observe(value: Value, endpoints_empty: bool) -> Result<()> {
    let mut bridge = locked()?;
    bridge.observed = Some(value);
    bridge.endpoints_empty = endpoints_empty;
    Ok(())
}

pub(super) fn request() -> Result<Option<Request>> {
    let bridge = locked()?;
    Ok(bridge
        .renderer_failure
        .is_none()
        .then(|| bridge.active.clone())
        .flatten())
}

pub(super) fn is_current(id: RequestId) -> bool {
    locked().is_ok_and(|bridge| {
        bridge.renderer_failure.is_none()
            && bridge.active.as_ref().is_some_and(|active| active.id == id)
    })
}

pub(super) fn pending(id: RequestId, receipt: Value) -> Result<()> {
    let mut bridge = locked()?;
    if bridge.active.as_ref().is_some_and(|active| active.id == id) {
        bridge.state = json!({"state":"readback_pending","request":id,"prerequisite":receipt});
    }
    Ok(())
}

pub(super) fn complete(id: RequestId, receipt: Value, image: CapturedPng) -> Result<()> {
    let mut bridge = locked()?;
    if !bridge.active.as_ref().is_some_and(|active| active.id == id) {
        return Ok(());
    }
    ensure!(
        bridge.renderer_failure.is_none() && bridge.image.is_none(),
        "failed or duplicate arrival readback"
    );
    bridge.state = json!({"state":"captured","request":id,"receipt":receipt,
        "width":image.width,"height":image.height,"png_bytes":image.bytes.len(),"png_blake3":image.blake3});
    bridge.image = Some(image);
    Ok(())
}

pub(super) fn fail(id: RequestId, error: impl ToString) {
    if let Ok(mut bridge) = locked()
        && bridge.active.as_ref().is_some_and(|active| active.id == id)
    {
        bridge.state = json!({"state":"failed","request":id,"error":error.to_string()});
        bridge.image = None;
    }
}

fn locked() -> Result<std::sync::MutexGuard<'static, Bridge>> {
    BRIDGE
        .get_or_init(|| Mutex::new(Bridge::default()))
        .lock()
        .map_err(|_| anyhow::anyhow!("arrival observation bridge is unavailable"))
}
