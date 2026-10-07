//! Orbit, pan and zoom from the pointer over the view.

use eframe::egui::{PointerButton, Response, Ui};
use fabelgeist_botany::viewer::camera::OrbitCamera;

/// Steer `camera` from what the pointer did over the view in `response`.
pub fn steer(ui: &Ui, response: &Response, camera: &mut OrbitCamera) {
    if response.dragged_by(PointerButton::Primary) {
        let delta = response.drag_delta();
        camera.rotate(delta.x, delta.y);
    }
    if response.dragged_by(PointerButton::Secondary) {
        let delta = response.drag_delta();
        camera.pan(delta.x, delta.y, response.rect.height());
    }
    if response.hovered() {
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll != 0.0 {
            // egui scrolls up in positive points, botany's camera closes in
            // on a negative wheel.
            camera.zoom(-scroll);
        }
    }
}
