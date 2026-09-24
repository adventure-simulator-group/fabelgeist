//! Standalone procedural-art browser viewer; no gameplay or network session.
#![expect(
    dead_code,
    reason = "the showcase reuses the production presentation module tree"
)]

mod art_demo;
mod camera;
mod presentation;
mod weapon_preview_material;

#[cfg(target_family = "wasm")]
fn main() {
    std::panic::set_hook(Box::new(|info| {
        art_demo::renderer_failed();
        console_error_panic_hook::hook(info);
    }));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    art_demo::run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn boot() {
    art_demo::run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn command(json: &str) -> Result<(), String> {
    art_demo::queue(json)
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn status() -> String {
    art_demo::status()
}
