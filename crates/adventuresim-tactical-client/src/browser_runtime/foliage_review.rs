//! Opt-in HTTP-byte admission and request-owned browser foliage evidence.
//! This diagnostic uses the existing app/device and never joins a server.

use adventuresim_environment::render::{
    benchmark::{CompletionPolicy, ForestBenchmark, ForestRequestId, ValidationPolicy},
    trees::{TreeBatching, TreeForestPlugin},
};
use bevy::prelude::*;
use std::num::NonZeroU64;
use web_time::Instant;

pub(crate) use bridge::{Command, renderer_failed};

mod bridge;
mod capture;
mod fault;
mod inventory;
mod scene;
mod session;
mod wire;

pub(crate) struct FoliageReviewPlugin;

impl Plugin for FoliageReviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TreeForestPlugin)
            .insert_resource(TreeBatching::RegionalCells)
            .init_resource::<session::Session>()
            .add_systems(Update, session::advance)
            .add_systems(
                PostUpdate,
                (scene::isolate, fault::apply)
                    .chain()
                    .after(crate::presentation::ownership::PresentationCameraSynchronization)
                    .before(bevy::camera::CameraUpdateSystems)
                    .before(bevy::transform::TransformSystems::Propagate)
                    .before(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
            );
        ForestBenchmark::new(
            Instant::now(),
            ValidationPolicy::SamplingEndpoints,
            ForestRequestId::from_ordinal(NonZeroU64::MIN),
            CompletionPolicy::Queued,
        )
        .install_shared(app);
    }
}

pub(crate) fn apply(world: &mut World, command: Command) {
    world.resource_scope(|world, mut session: Mut<session::Session>| {
        if let Err(error) = session.apply(world, command) {
            session.fail(error.to_string());
        }
    });
}
