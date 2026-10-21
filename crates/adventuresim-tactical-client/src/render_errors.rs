//! Renderer failure policy shared by normal play and diagnostic consumers.

use bevy::render::error_handler::{RenderErrorHandler, RenderErrorPolicy};

/// Keeps rendering after a pipeline fails validation instead of quitting.
///
/// Browser WebGPU rejects some pipelines the native backend accepts (shader
/// modules and limits it does not expose). Bevy's default handler hard-quits
/// the app on the first such error, blanking the canvas. Ignoring it lets the
/// rest of the scene draw -- the instanced grass has its own pipeline -- and
/// surfaces every incompatible pipeline rather than only the first. Native
/// compiles every pipeline, so this never fires there.
pub(crate) fn resilient_render_errors() -> RenderErrorHandler {
    RenderErrorHandler(|_error, _main_world, _render_world| {
        #[cfg(all(target_family = "wasm", feature = "foliage-review"))]
        crate::browser_runtime::foliage_review::renderer_failed();
        RenderErrorPolicy::Ignore
    })
}
