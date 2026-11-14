//! Native backend: silent. Native builds serve development and profiling;
//! sound plays only in the browser client.

pub(super) fn preload(_asset_path: &str) {}

pub(super) fn play_spatial(_asset_path: &str, _left: f32, _right: f32, _speed: f32) {}

pub(super) fn set_loop_volume(_asset_path: &str, _volume: f32) {}
