use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn wasm_generation_jobs(input_json: &str) -> Result<String, JsValue> {
    let jobs = super::jobs(input_json).map_err(|error| JsValue::from_str(&error))?;
    serde_json::to_string(&jobs).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn wasm_begin_generation() {
    let mut products = super::products();
    products.scenes.clear();
    products.facades.clear();
    products.placements.clear();
}

#[wasm_bindgen]
pub fn wasm_generate_job(job_json: &str, dependencies: &[u8]) -> Result<Vec<u8>, JsValue> {
    super::generate(job_json, dependencies).map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub fn wasm_generation_dependencies(job_json: &str) -> Result<Vec<u8>, JsValue> {
    super::dependencies(job_json).map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub fn wasm_venue_jobs(input_json: &str, view_json: &str) -> Result<String, JsValue> {
    let jobs =
        super::venue_jobs(input_json, view_json).map_err(|error| JsValue::from_str(&error))?;
    serde_json::to_string(&jobs).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn wasm_receive_job(job_json: &str, bytes: &[u8]) -> Result<(), JsValue> {
    super::receive(job_json, bytes).map_err(|error| JsValue::from_str(&error))
}
