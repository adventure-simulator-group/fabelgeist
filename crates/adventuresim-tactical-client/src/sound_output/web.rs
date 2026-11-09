//! Web Audio backend: the `window.sfx*` functions defined by `static/sfx.js`.
//!
//! Each call is one wasm-to-JavaScript hop. The page collects a frame's calls
//! and posts them to its AudioWorklet mixer in one message.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = sfxPreload)]
    pub(super) fn preload(asset_path: &str);

    #[wasm_bindgen(js_namespace = window, js_name = sfxPlay)]
    pub(super) fn play_spatial(asset_path: &str, left: f32, right: f32, speed: f32);

    #[wasm_bindgen(js_namespace = window, js_name = sfxLoop)]
    pub(super) fn set_loop_volume(asset_path: &str, volume: f32);
}
