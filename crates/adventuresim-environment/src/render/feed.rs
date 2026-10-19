//! Snapshots on their way to the screen. Copying and sorting a grown
//! forest's few hundred thousand records takes tens of milliseconds, so a
//! fed snapshot is made ready on a background task, and drawn in one frame
//! once it is. Snapshots fed meanwhile replace each other: only the latest
//! is drawn.

use std::sync::Arc;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future::{block_on, poll_once};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use fabelgeist_botany::viewer::sim::Snapshot;

use super::benchmark::ForestBenchmark;
use super::{ForestDetail, chunks, ground, levels, materials, records};

/// The snapshot to draw next, if a new one arrived, and the one being made
/// ready.
#[derive(Resource, Default)]
pub struct ForestFeed {
    pending: Option<Arc<Snapshot>>,
    preparing: Option<Task<Prepared>>,
}

impl ForestFeed {
    /// Draw `snapshot` as soon as it is made ready, off the main thread.
    pub fn show(&mut self, snapshot: impl Into<Arc<Snapshot>>) {
        self.pending = Some(snapshot.into());
    }

    /// Whether every snapshot fed so far is drawn.
    pub fn is_drawn(&self) -> bool {
        self.pending.is_none() && self.preparing.is_none()
    }
}

/// A snapshot made ready to draw: its records' bytes, its instances by
/// chunk, and the ground it brings.
struct Prepared {
    records: records::Records,
    chunks: chunks::Chunks,
    ground: ground::Ground,
}

/// Start making the latest fed snapshot ready, unless one already is being
/// made ready.
pub(super) fn prepare(mut feed: ResMut<ForestFeed>) {
    if feed.preparing.is_some() {
        return;
    }
    let Some(snapshot) = feed.pending.take() else {
        return;
    };
    feed.preparing = Some(AsyncComputeTaskPool::get().spawn(async move {
        Prepared {
            records: records::Records::new(&snapshot),
            chunks: chunks::Chunks::new(&snapshot),
            ground: ground::Ground::new(&snapshot),
        }
    }));
}

/// What new chunks' level of detail follows: the camera and the detail
/// asked for.
#[derive(SystemParam)]
pub(super) struct Sight<'w> {
    view: Res<'w, levels::ForestLodView>,
    detail: Res<'w, ForestDetail>,
    probe: Option<Res<'w, ForestBenchmark>>,
}

/// Draw a snapshot made ready: its records into the storage buffers, its
/// chunks as entities, and its terrain and buildings when it brings them.
pub(super) fn draw(
    mut commands: Commands,
    mut feed: ResMut<ForestFeed>,
    mut forest: ResMut<materials::ForestMaterials>,
    mut stores: materials::ForestStores,
    mut ground: ground::GroundAssets,
    mut shown: chunks::Shown,
    sight: Sight,
) {
    let Some(task) = feed.preparing.as_mut() else {
        return;
    };
    let Some(prepared) = block_on(poll_once(task)) else {
        return;
    };
    feed.preparing = None;
    forest.store(prepared.records, &mut stores);
    let placement = chunks::Placement {
        forest: &forest,
        view: &sight.view,
        detail: &sight.detail,
    };
    chunks::show(
        &mut commands,
        &placement,
        prepared.chunks,
        &mut shown,
        sight.probe.as_deref(),
    );
    ground.show(&mut commands, &prepared.ground);
}
