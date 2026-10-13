//! Opaque preparation tickets cross JavaScript; Rust admits owner and identity.
use super::{GenerationOwner, PreparationError, PreparationTicket};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn wasm_begin_generation(owner_json: &str) -> Result<String, JsValue> {
    let owner: GenerationOwner = decode(owner_json)?;
    encode(&super::begin(owner).map_err(JsValue::from)?)
}

#[wasm_bindgen]
pub fn wasm_finish_generation(ticket_json: &str, input_json: &str) -> Result<(), JsValue> {
    super::finish(decode(ticket_json)?, &decode(input_json)?).map_err(JsValue::from)
}

#[wasm_bindgen]
pub fn wasm_cancel_generation(ticket_json: &str) -> Result<(), JsValue> {
    super::cancel(decode(ticket_json)?).map_err(JsValue::from)
}

#[wasm_bindgen]
pub fn wasm_landscape_jobs(
    ticket_json: &str,
    input_json: &str,
    graphics: &str,
) -> Result<String, JsValue> {
    encode(
        &super::landscape::jobs(decode(ticket_json)?, input_json, graphics)
            .map_err(JsValue::from)?,
    )
}

#[wasm_bindgen]
pub fn wasm_generation_jobs(ticket_json: &str, input_json: &str) -> Result<String, JsValue> {
    encode(&super::jobs(decode(ticket_json)?, input_json).map_err(JsValue::from)?)
}

#[wasm_bindgen]
pub fn wasm_generate_job(job_json: &str, dependencies: &[u8]) -> Result<Vec<u8>, JsValue> {
    super::generate(job_json, dependencies).map_err(JsValue::from)
}

#[wasm_bindgen]
pub fn wasm_generation_dependencies(ticket_json: &str, job_json: &str) -> Result<Vec<u8>, JsValue> {
    super::dependencies(decode::<PreparationTicket>(ticket_json)?, job_json).map_err(JsValue::from)
}

#[wasm_bindgen]
pub fn wasm_venue_jobs(
    ticket_json: &str,
    input_json: &str,
    view_json: &str,
) -> Result<String, JsValue> {
    encode(&super::venue_jobs(decode(ticket_json)?, input_json, view_json).map_err(JsValue::from)?)
}

#[wasm_bindgen]
pub fn wasm_receive_job(ticket_json: &str, job_json: &str, bytes: &[u8]) -> Result<(), JsValue> {
    super::receive(decode(ticket_json)?, job_json, bytes).map_err(JsValue::from)
}

fn decode<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, JsValue> {
    serde_json::from_str(json).map_err(|error| JsValue::from(PreparationError::from(error)))
}

fn encode(value: &impl serde::Serialize) -> Result<String, JsValue> {
    serde_json::to_string(value).map_err(|error| JsValue::from(PreparationError::from(error)))
}
