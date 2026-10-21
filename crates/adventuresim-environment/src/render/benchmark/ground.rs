//! Observe successful ordinary ground draws for the active capture camera.
//! A sky-only or floating-forest readback cannot establish area arrival.

use std::{collections::HashSet, sync::Mutex};

use bevy::camera::visibility::{VisibilitySystems, VisibleEntities};
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::ecs::system::{SystemParamItem, lifetimeless::SRes};
use bevy::pbr::DrawMaterial;
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::render_phase::{
    DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderCommandState,
    TrackedRenderPass,
};
use bevy::render::{
    sync_world::MainEntity,
    view::{ExtractedView, RetainedViewEntity},
};

use super::super::{ForestViewCamera, ground::GroundPart};
use super::{ForestBenchmark, ForestRequestId};

type CaptureCameras<'world, 'state> = Query<
    'world,
    'state,
    (Entity, &'static Camera, &'static VisibleEntities),
    (With<Camera3d>, With<ForestViewCamera>),
>;

/// Native entity membership for one active capture view, reset on extraction.
#[derive(Resource, Default)]
pub(super) struct GroundFrame {
    request: Option<ForestRequestId>,
    camera: Option<RetainedViewEntity>,
    expected: HashSet<MainEntity>,
    drawn: Mutex<HashSet<MainEntity>>,
}

#[derive(Resource, Default)]
pub(super) struct VisibilityRequest(Option<ForestRequestId>);

struct RecordGround;

impl GroundFrame {
    pub(super) fn view(&self) -> Option<RetainedViewEntity> {
        self.camera
    }

    pub(super) fn owns_view(&self, view: RetainedViewEntity) -> bool {
        self.camera == Some(view)
    }

    pub(super) fn complete(&self, probe: &ForestBenchmark) -> bool {
        let complete = self.request == Some(probe.request_id())
            && self.camera.is_some()
            && self
                .drawn
                .lock()
                .is_ok_and(|drawn| self.expected.is_subset(&drawn));
        if complete && let Ok(mut ground) = probe.inner.ground.lock() {
            *ground = self
                .expected
                .iter()
                .map(|entity| entity.id().to_bits())
                .collect();
        }
        complete
    }
}

pub(super) fn observe_visibility(app: &mut App) {
    app.init_resource::<VisibilityRequest>().add_systems(
        PostUpdate,
        mark_visibility.after(VisibilitySystems::CheckVisibility),
    );
}

fn mark_visibility(probe: Res<ForestBenchmark>, mut visibility: ResMut<VisibilityRequest>) {
    visibility.0 = Some(probe.request_id());
}

impl<P: PhaseItem> RenderCommand<P> for RecordGround {
    type Param = SRes<GroundFrame>;
    type ViewQuery = &'static ExtractedView;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        view: &'w ExtractedView,
        _ground: Option<()>,
        frame: SystemParamItem<'w, '_, Self::Param>,
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let frame = frame.into_inner();
        if frame.camera == Some(view.retained_view_entity)
            && frame.expected.contains(&item.main_entity())
            && let Ok(mut drawn) = frame.drawn.lock()
        {
            drawn.insert(item.main_entity());
        }
        RenderCommandResult::Success
    }
}

pub(super) fn install(world: &mut World) {
    world.init_resource::<GroundFrame>();
    let command = RenderCommandState::<Opaque3d, (DrawMaterial, RecordGround)>::new(world);
    world
        .resource::<DrawFunctions<Opaque3d>>()
        .write()
        .add_with::<DrawMaterial, _>(command);
}

pub(super) fn extract(
    mut frame: ResMut<GroundFrame>,
    views: Extract<CaptureCameras>,
    surfaces: Extract<Query<&GroundPart>>,
    probe: Extract<Res<ForestBenchmark>>,
    visibility: Extract<Res<VisibilityRequest>>,
) {
    frame.request = None;
    frame.camera = None;
    frame.expected.clear();
    if let Ok(mut drawn) = frame.drawn.lock() {
        drawn.clear();
    }
    if visibility.0 != Some(probe.request_id()) {
        return;
    }
    let mut active = views.iter().filter(|(_, camera, _)| camera.is_active);
    let Some((camera, _, visible)) = active.next() else {
        return;
    };
    if active.next().is_some() {
        return;
    }
    frame.camera = Some(RetainedViewEntity::new(camera.into(), None, 0));
    frame.request = Some(probe.request_id());
    for entity in visible.get(std::any::TypeId::of::<Mesh3d>()) {
        if surfaces.contains(*entity) {
            frame.expected.insert((*entity).into());
        }
    }
}
