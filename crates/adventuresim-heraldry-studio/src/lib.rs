//! One editor and material scene shared by native and WebGPU targets.
mod app;
mod baking;
mod exchange;
#[cfg(not(target_family = "wasm"))]
pub mod review;
mod scene;
mod ui;
pub use app::run;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_studio() {
    console_error_panic_hook::set_once();
    run();
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn bake_heraldry(source: &str) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    baking::execute(source).map_err(|e| wasm_bindgen::JsValue::from_str(&e))
}

/// Portable deterministic recipe search for workers and external tools.
pub fn solve_paint(source: &str) -> Result<String, adventuresim_heraldry::Error> {
    let request: adventuresim_heraldry::paint::mixing::SearchRequest =
        serde_json::from_str(source)?;
    Ok(serde_json::to_string(&request.solve()?)?)
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn solve_heraldry_paint(source: &str) -> Result<String, wasm_bindgen::JsValue> {
    solve_paint(source).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
}
