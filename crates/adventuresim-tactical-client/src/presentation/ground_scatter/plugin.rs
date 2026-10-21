//! Install ground-cover renderers, cell visibility and retained prototypes.

use super::{
    GroundFoliagePresentationCache, InstancedGrassPlugin, WoodyUnderstoryPresentationCache,
    gardens::GardenPresentationPlugin, litter::SharedLitterPlugin,
};
use bevy::prelude::*;

pub(in crate::presentation) struct GroundScatterPlugin;

impl Plugin for GroundScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InstancedGrassPlugin,
            SharedLitterPlugin,
            GardenPresentationPlugin,
        ))
        .init_resource::<WoodyUnderstoryPresentationCache>()
        .init_resource::<GroundFoliagePresentationCache>();
    }
}
