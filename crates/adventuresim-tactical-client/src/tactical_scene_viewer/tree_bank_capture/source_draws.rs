//! Actual successful production material commands for one fixed source pose.
//! Queue completion is a prefix prerequisite, not an exact screenshot timestamp.

use super::super::capture_state::{CapturePhase, SceneCaptureState};
use super::*;
use crate::presentation::{TacticalGameplayCamera, TreeSourcePart};
use bevy::{
    camera::visibility::{VisibilitySystems, VisibleEntities},
    core_pipeline::core_3d::{AlphaMask3d, Opaque3d},
    ecs::system::{SystemParamItem, lifetimeless::SRes},
    pbr::DrawMaterial,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        render_phase::{
            DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderCommandState,
            TrackedRenderPass,
        },
        renderer::RenderQueue,
        sync_world::MainEntity,
        view::{ExtractedView, RetainedViewEntity},
    },
};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

type SourceCameras<'world, 'state> = Query<
    'world,
    'state,
    (Entity, &'static Camera, &'static VisibleEntities),
    With<TacticalGameplayCamera>,
>;

#[derive(Resource, Clone, Default)]
pub(super) struct SourceDraws(Arc<Mutex<BTreeMap<usize, Arc<Pose>>>>);

#[derive(Clone, Copy, PartialEq, Eq)]
struct Member {
    entity: MainEntity,
    part: TreeSourcePart,
}

struct Pose {
    view: usize,
    camera: RetainedViewEntity,
    camera_bits: u64,
    expected: Vec<Member>,
    drawn: Mutex<HashSet<MainEntity>>,
    completed_drawn: Mutex<Option<Vec<u64>>>,
    invalid: AtomicBool,
    queued: AtomicBool,
    complete: AtomicBool,
}

#[derive(Resource, Default)]
struct Frame(Option<Arc<Pose>>);

#[derive(Resource, Default)]
struct VisibilityMark(Option<usize>);

struct Record;

impl SourceDraws {
    pub(super) fn install(app: &mut App) -> Self {
        let draws = Self::default();
        app.insert_resource(draws.clone())
            .init_resource::<VisibilityMark>()
            .add_systems(PostUpdate, mark.after(VisibilitySystems::CheckVisibility));
        let render = app.sub_app_mut(RenderApp);
        render
            .insert_resource(draws.clone())
            .init_resource::<Frame>();
        let world = render.world_mut();
        let opaque = RenderCommandState::<Opaque3d, (DrawMaterial, Record)>::new(world);
        world
            .resource::<DrawFunctions<Opaque3d>>()
            .write()
            .add_with::<DrawMaterial, _>(opaque);
        let alpha = RenderCommandState::<AlphaMask3d, (DrawMaterial, Record)>::new(world);
        world
            .resource::<DrawFunctions<AlphaMask3d>>()
            .write()
            .add_with::<DrawMaterial, _>(alpha);
        render
            .add_systems(ExtractSchedule, extract)
            .add_systems(Render, complete.in_set(RenderSystems::Cleanup));
        draws
    }

    pub(super) fn proof(&self, view: usize) -> Option<serde_json::Value> {
        let poses = self.0.lock().ok()?;
        let pose = poses.get(&view)?;
        if !pose.complete.load(Ordering::Acquire) || pose.invalid.load(Ordering::Acquire) {
            return None;
        }
        let drawn = pose.completed_drawn.lock().ok()?;
        Some(
            serde_json::json!({"role": Role::Source, "view": pose.view, "camera_entity_bits": pose.camera_bits,
            "expected": pose.expected.iter().map(|member| serde_json::json!({"entity_bits": member.entity.id().to_bits(), "part": member.part})).collect::<Vec<_>>(),
            "successful_draw_entity_bits": &*drawn, "queue_prefix_complete": true,
            "scope": "successful DrawMaterial commands in one actual opaque/alpha-mask render frame for visible source members; later screenshots remain held at this pose"}),
        )
    }

    fn invalidate(&self, view: usize) {
        if let Ok(poses) = self.0.lock()
            && let Some(pose) = poses.get(&view)
        {
            pose.invalid.store(true, Ordering::Release);
        }
    }

    pub(super) fn all_valid(&self, count: usize) -> bool {
        self.0.lock().is_ok_and(|poses| {
            poses.len() == count
                && poses.values().all(|pose| {
                    pose.complete.load(Ordering::Acquire) && !pose.invalid.load(Ordering::Acquire)
                })
        })
    }
}

fn mark(
    state: Option<Res<SceneCaptureState>>,
    cameras: Query<(&Camera, &GlobalTransform, &Projection), With<TacticalGameplayCamera>>,
    mut mark: ResMut<VisibilityMark>,
    draws: Res<SourceDraws>,
    depth: Res<cameras::PerspectiveDepth>,
) {
    mark.0 = None;
    let Some(state) = state else {
        return;
    };
    if state.phase == CapturePhase::Configure {
        return;
    }
    let Some(view) = state.views.get(state.view) else {
        return;
    };
    let CapturePose::Bank(expected) = view.pose else {
        return;
    };
    let Ok((camera, transform, projection)) = cameras.single() else {
        draws.invalidate(state.view);
        return;
    };
    if expected.matches(camera, transform, projection, &depth) {
        mark.0 = Some(state.view);
    } else {
        draws.invalidate(state.view);
    }
}

fn extract(
    mut frame: ResMut<Frame>,
    draws: Res<SourceDraws>,
    state: Extract<Option<Res<SceneCaptureState>>>,
    mark: Extract<Res<VisibilityMark>>,
    cameras: Extract<SourceCameras>,
    parts: Extract<Query<&TreeSourcePart>>,
) {
    frame.0 = None;
    let Some(state) = state.as_ref() else {
        return;
    };
    if mark.0 != Some(state.view) {
        draws.invalidate(state.view);
        return;
    }
    let Ok((camera, active, visible)) = cameras.single() else {
        return;
    };
    if !active.is_active {
        draws.invalidate(state.view);
        return;
    }
    let Some(view) = state.views.get(state.view) else {
        return;
    };
    let CapturePose::Bank(camera_pose) = view.pose else {
        return;
    };
    let mut expected: Vec<_> = visible
        .get(std::any::TypeId::of::<Mesh3d>())
        .iter()
        .filter_map(|entity| {
            parts.get(*entity).ok().map(|part| Member {
                entity: (*entity).into(),
                part: *part,
            })
        })
        .collect();
    expected.sort_by_key(|member| member.entity.id().to_bits());
    if !expected
        .iter()
        .any(|member| member.part == TreeSourcePart::Wood)
        || (camera_pose.plate != cameras::Plate::Root
            && !expected
                .iter()
                .any(|member| member.part == TreeSourcePart::Foliage))
    {
        draws.invalidate(state.view);
        return;
    }
    let Ok(mut poses) = draws.0.lock() else {
        return;
    };
    let camera_id = RetainedViewEntity::new(camera.into(), None, 0);
    let pose = poses.entry(state.view).or_insert_with(|| {
        Arc::new(Pose {
            view: state.view,
            camera: camera_id,
            camera_bits: camera.to_bits(),
            expected: expected.clone(),
            drawn: Mutex::new(HashSet::new()),
            completed_drawn: Mutex::new(None),
            invalid: AtomicBool::new(false),
            queued: AtomicBool::new(false),
            complete: AtomicBool::new(false),
        })
    });
    // Source geometry, visibility and camera are held for this view. A later
    // frame cannot silently replace a receipt's source camera or membership.
    if pose.camera != camera_id || pose.expected != expected {
        pose.invalid.store(true, Ordering::Release);
    }
    if !pose.queued.load(Ordering::Acquire) {
        if let Ok(mut drawn) = pose.drawn.lock() {
            drawn.clear();
        } else {
            pose.invalid.store(true, Ordering::Release);
        }
    }
    frame.0 = Some(Arc::clone(pose));
}

impl<P: PhaseItem> RenderCommand<P> for Record {
    type Param = SRes<Frame>;
    type ViewQuery = &'static ExtractedView;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        view: &'w ExtractedView,
        _entity: Option<()>,
        frame: SystemParamItem<'w, '_, Self::Param>,
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(pose) = &frame.into_inner().0
            && pose.camera == view.retained_view_entity
            && pose
                .expected
                .iter()
                .any(|member| member.entity == item.main_entity())
            && !pose.queued.load(Ordering::Acquire)
        {
            if let Ok(mut drawn) = pose.drawn.lock() {
                drawn.insert(item.main_entity());
            } else {
                pose.invalid.store(true, Ordering::Release);
            }
        }
        RenderCommandResult::Success
    }
}

fn complete(frame: Res<Frame>, queue: Res<RenderQueue>) {
    let Some(pose) = &frame.0 else {
        return;
    };
    if pose.invalid.load(Ordering::Acquire) || pose.queued.load(Ordering::Acquire) {
        return;
    }
    let Ok(drawn) = pose.drawn.lock() else {
        pose.invalid.store(true, Ordering::Release);
        return;
    };
    if !pose
        .expected
        .iter()
        .all(|member| drawn.contains(&member.entity))
    {
        return;
    }
    let mut submitted: Vec<_> = drawn.iter().map(|entity| entity.id().to_bits()).collect();
    submitted.sort_unstable();
    pose.queued.store(true, Ordering::Release);
    let pose = Arc::clone(pose);
    queue.on_submitted_work_done(move || {
        if let Ok(mut completed) = pose.completed_drawn.lock() {
            *completed = Some(submitted);
        } else {
            pose.invalid.store(true, Ordering::Release);
        }
        pose.complete.store(true, Ordering::Release);
    });
}
