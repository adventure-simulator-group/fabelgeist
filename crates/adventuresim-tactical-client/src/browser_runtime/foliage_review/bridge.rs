//! The existing browser command queue carries bytes only after request start.

use super::{
    fault::Fault,
    wire::{Document, MAX_BANK_BYTES, MAX_POPULATION_BYTES},
};
use adventuresim_environment::render::benchmark::ForestRequestId;
use anyhow::{Result, bail, ensure};
use std::{
    num::NonZeroU64,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};
use wasm_bindgen::prelude::*;
use web_time::Instant;

static BRIDGE: OnceLock<Mutex<Bridge>> = OnceLock::new();
static RENDER_FAILED: AtomicBool = AtomicBool::new(false);

struct Bridge {
    last: ForestRequestId,
    active: Option<ForestRequestId>,
    submitted: bool,
    status: String,
    images: Vec<Vec<u8>>,
}

pub(crate) enum Command {
    Begin {
        request: ForestRequestId,
        document: Box<Document>,
        started: Instant,
    },
    Payload {
        request: ForestRequestId,
        bank: Vec<u8>,
        population: Vec<u8>,
    },
    Fault {
        request: ForestRequestId,
        fault: Fault,
    },
    Failure {
        request: ForestRequestId,
        message: String,
    },
}

impl Default for Bridge {
    fn default() -> Self {
        Self {
            last: ForestRequestId::FIRST,
            active: None,
            submitted: false,
            status: String::new(),
            images: Vec::new(),
        }
    }
}

#[wasm_bindgen]
pub fn wasm_foliage_begin(json: String) -> Result<u32, JsValue> {
    begin(&json).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn wasm_foliage_payload(id: u32, bank: Vec<u8>, population: Vec<u8>) -> Result<(), JsValue> {
    payload(id, bank, population).map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Apply one bounded diagnostic ownership fault to the current arrival.
#[wasm_bindgen]
pub fn wasm_foliage_fault(id: u32, json: String) -> Result<(), JsValue> {
    let result = (|| -> Result<()> {
        let fault = Fault::parse(&json)?;
        let bridge = locked()?;
        let request = admit_request(id)?;
        ensure!(
            bridge.active == Some(request),
            "fault for an inactive foliage request"
        );
        super::super::queue_foliage(Command::Fault { request, fault }).map_err(anyhow::Error::msg)
    })();
    result.map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Fetch, device-loss and uncaptured WebGPU errors are sticky process failures.
#[wasm_bindgen]
pub fn wasm_foliage_failure(id: u32, message: String) -> Result<(), JsValue> {
    let result = (|| -> Result<()> {
        let bridge = locked()?;
        let request = admit_request(id)?;
        let Some(active) = bridge.active.filter(|active| *active == request) else {
            bail!("failure for an inactive foliage request");
        };
        super::super::queue_foliage(Command::Failure {
            request: active,
            message,
        })
        .map_err(anyhow::Error::msg)
    })();
    result.map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn wasm_foliage_status() -> String {
    locked()
        .map(|bridge| bridge.status.clone())
        .unwrap_or_else(|error| {
            serde_json::json!({"state":"failed","error":error.to_string()}).to_string()
        })
}

/// PNG transport occurs after the readback timestamp, outside arrival timing.
#[wasm_bindgen]
pub fn wasm_foliage_image(index: usize) -> Result<Vec<u8>, JsValue> {
    locked()
        .and_then(|bridge| {
            bridge
                .images
                .get(index)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("unknown foliage image"))
        })
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Call from the WebGPU device's loss and uncaptured-error listeners.
#[wasm_bindgen]
pub fn wasm_foliage_device_failed() {
    renderer_failed();
}

pub(super) fn booted() {
    if let Ok(mut bridge) = locked()
        && bridge.status.is_empty()
    {
        bridge.status = serde_json::json!({"state":"idle"}).to_string();
    }
}

pub(crate) fn renderer_failed() {
    RENDER_FAILED.store(true, Ordering::Release);
}
pub(super) fn has_render_failure() -> bool {
    RENDER_FAILED.load(Ordering::Acquire)
}

pub(super) fn publish(status: serde_json::Value, terminal: bool) {
    if let Ok(mut bridge) = locked() {
        bridge.status = status.to_string();
        if terminal {
            bridge.active = None;
        }
    }
}

pub(super) fn push_image(bytes: Vec<u8>) -> Result<()> {
    locked()?.images.push(bytes);
    Ok(())
}

fn locked() -> Result<std::sync::MutexGuard<'static, Bridge>> {
    BRIDGE
        .get_or_init(|| Mutex::new(Bridge::default()))
        .lock()
        .map_err(|_| anyhow::anyhow!("foliage review bridge unavailable"))
}

fn admit_request(id: u32) -> Result<ForestRequestId> {
    NonZeroU64::new(u64::from(id))
        .map(ForestRequestId::from_ordinal)
        .ok_or_else(|| anyhow::anyhow!("zero foliage request"))
}

fn begin(json: &str) -> Result<u32> {
    let started = Instant::now();
    let document = Document::parse(json)?;
    let mut bridge = locked()?;
    ensure!(
        bridge.active.is_none(),
        "previous foliage request is still active"
    );
    // FIRST belongs only to the plugin's bootstrap probe. Real arrivals carry
    // the same identifier through the bridge, renderer, callbacks and reports.
    let next = bridge
        .last
        .successor()
        .ok_or_else(|| anyhow::anyhow!("foliage request ordinals exhausted"))?;
    let wire_ordinal = u32::try_from(next.ordinal().get())?;
    super::super::queue_foliage(Command::Begin {
        request: next,
        document: Box::new(document),
        started,
    })
    .map_err(anyhow::Error::msg)?;
    bridge.last = next;
    bridge.active = Some(next);
    bridge.submitted = false;
    bridge.images.clear();
    bridge.status = serde_json::json!({"state":"fetching", "request":next}).to_string();
    Ok(wire_ordinal)
}

fn payload(id: u32, bank: Vec<u8>, population: Vec<u8>) -> Result<()> {
    let mut bridge = locked()?;
    let id = admit_request(id)?;
    ensure!(
        bridge.active == Some(id) && !bridge.submitted,
        "stale or duplicate foliage payload"
    );
    ensure!(
        bank.len() <= MAX_BANK_BYTES && population.len() <= MAX_POPULATION_BYTES,
        "oversized foliage payload"
    );
    super::super::queue_foliage(Command::Payload {
        request: id,
        bank,
        population,
    })
    .map_err(anyhow::Error::msg)?;
    bridge.submitted = true;
    Ok(())
}
