//! Browser-shaped scheduling for native Tracy profiles.
//!
//! The browser client runs every schedule on one thread and renders inline,
//! because wasm has no `multi_threaded` executor and no pipelined rendering.
//! A native profile measured with the multithreaded executor hides that cost
//! behind 32 cores and adds executor overhead the browser never pays, so the
//! `tracy` build runs native the way the browser does.

use bevy::{
    app::{Plugin, PluginGroupBuilder},
    ecs::schedule::SingleThreadedExecutor,
    prelude::*,
    render::pipelined_rendering::PipelinedRenderingPlugin,
};

/// Renders inline on the main thread instead of on a pipelined render thread,
/// and runs every schedule on one thread, as the browser does.
pub(crate) fn browser_shaped(plugins: PluginGroupBuilder) -> PluginGroupBuilder {
    plugins
        .disable::<PipelinedRenderingPlugin>()
        .add(SingleThreadedSchedulesPlugin)
}

/// Replaces the executor of every schedule in every sub-app with
/// [`SingleThreadedExecutor`] once all plugins have added their schedules.
struct SingleThreadedSchedulesPlugin;

impl Plugin for SingleThreadedSchedulesPlugin {
    fn build(&self, _app: &mut App) {}

    fn cleanup(&self, app: &mut App) {
        for sub_app in app.sub_apps_mut().iter_mut() {
            if let Some(mut schedules) = sub_app.world_mut().get_resource_mut::<Schedules>() {
                for (_, schedule) in schedules.iter_mut() {
                    schedule.set_executor(SingleThreadedExecutor::new());
                }
            }
        }
    }
}
