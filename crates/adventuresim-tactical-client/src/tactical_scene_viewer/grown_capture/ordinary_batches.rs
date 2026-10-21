//! Held ordinary main-color command and actual GPU instance membership proof.
use super::super::capture_state::{CapturePhase, SceneCaptureState};
use super::*;
use bevy::{
    camera::visibility::VisibilitySystems,
    core_pipeline::core_3d::{AlphaMask3d, Opaque3d},
    mesh::MeshTag,
    pbr::DrawMaterial,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        render_phase::{DrawFunctions, RenderCommandState},
        view::RetainedViewEntity,
    },
};
use controls::BatchControl;
use members::TaggedMember;
use readiness::{Readiness, ReadinessStage};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

mod bindings;
mod commands;
mod decode;
mod gather;
mod gpu_commands;
mod inbox;
pub(super) mod inventory;
mod members;
mod proof;
mod readbacks;
mod readiness;
mod snapshots;
mod source_buffers;
mod tags;
mod types;
mod validate;

const MAX_FIXTURE_MEMBERS: usize = 4096;

#[derive(Resource, Clone)]
pub(super) struct OrdinaryBatches(Arc<Shared>);

struct Shared {
    poses: Mutex<BTreeMap<usize, Arc<Pose>>>,
    tags: Mutex<HashMap<Entity, MeshTag>>,
    tag_capacity: Mutex<tags::TagCapacity>,
    retained_tags: Mutex<tags::RetainedTaggedMembers>,
    control: BatchControl,
}

struct Pose {
    view: usize,
    subject: Subject,
    first_frame: types::RenderFrameId,
    readiness: Readiness,
    selection: Option<Arc<generated::selection::CheckpointSelection>>,
    buffers: Vec<inventory::SourceBinding>,
    camera: RetainedViewEntity,
    camera_bits: u64,
    expected: Vec<TaggedMember>,
    output: PathBuf,
    invalid: AtomicBool,
    complete: AtomicBool,
    failure: Mutex<Option<String>>,
    submission: Mutex<Option<proof::Submission>>,
}

#[derive(Resource, Default)]
struct Frame {
    serial: types::RenderFrameId,
    pose: Option<Arc<Pose>>,
    commands: Mutex<Vec<commands::Receipt>>,
}

impl OrdinaryBatches {
    pub(super) fn install(app: &mut App, control: BatchControl) -> Self {
        let observer = Self(Arc::new(Shared {
            poses: Mutex::new(BTreeMap::new()),
            tags: Mutex::new(HashMap::new()),
            tag_capacity: Mutex::new(tags::TagCapacity::default()),
            retained_tags: Mutex::new(tags::RetainedTaggedMembers::default()),
            control,
        }));
        app.insert_resource(observer.clone()).add_systems(
            PostUpdate,
            apply_control.before(VisibilitySystems::VisibilityPropagate),
        );
        let render = app.sub_app_mut(RenderApp);
        render
            .insert_resource(observer.clone())
            .init_resource::<Frame>()
            .init_resource::<readbacks::Queue>();
        let world = render.world_mut();
        let opaque = RenderCommandState::<Opaque3d, (DrawMaterial, commands::Record)>::new(world);
        world
            .resource::<DrawFunctions<Opaque3d>>()
            .write()
            .add_with::<DrawMaterial, _>(opaque);
        let alpha = RenderCommandState::<AlphaMask3d, (DrawMaterial, commands::Record)>::new(world);
        world
            .resource::<DrawFunctions<AlphaMask3d>>()
            .write()
            .add_with::<DrawMaterial, _>(alpha);
        render.add_systems(ExtractSchedule, extract).add_systems(
            Render,
            readbacks::map_submitted.in_set(RenderSystems::Cleanup),
        );
        gather::install(render);
        observer
    }

    pub(super) fn proof(&self, view: usize) -> Result<Option<serde_json::Value>> {
        let pose = self.pose(view)?;
        let Some(pose) = pose else {
            return Ok(None);
        };
        pose.poll()
    }

    pub(super) fn invalidate(&self, view: usize) {
        if let Ok(Some(pose)) = self.pose(view) {
            pose.fail("held camera or inventory changed");
        }
    }

    pub(super) fn inventory_verified(&self) -> bool {
        self.0.poses.lock().is_ok_and(|poses| {
            !poses.is_empty()
                && poses
                    .values()
                    .all(|pose| !pose.invalid.load(Ordering::Acquire))
        })
    }

    pub(super) fn all_valid(&self, count: usize) -> bool {
        self.0.poses.lock().is_ok_and(|poses| {
            poses.len() == count
                && poses.values().all(|pose| {
                    pose.complete.load(Ordering::Acquire) && !pose.invalid.load(Ordering::Acquire)
                })
        })
    }

    pub(super) fn check_capture(
        &self,
        view: usize,
        current: &inventory::CurrentInventory,
        tags: &Query<&MeshTag>,
    ) -> Result<()> {
        self.check_retained_tags(|entity| tags.get(entity).ok().cloned())?;
        let pose = self.pose(view)?.context("held GPU batch pose")?;
        ensure!(
            pose.complete.load(Ordering::Acquire) && !pose.invalid.load(Ordering::Acquire),
            "incomplete GPU batch proof"
        );
        let saved = self
            .0
            .tags
            .lock()
            .map_err(|_| anyhow::anyhow!("tree tag lock"))?;
        let members = TaggedMember::collect(current, &saved)?;
        ensure!(
            pose.expected == members,
            "GPU proof callback inventory drift"
        );
        for member in members {
            ensure!(
                tags.get(member.entity)?.0 == member.tag.0,
                "GPU proof callback tag drift"
            );
        }
        Ok(())
    }

    fn pose(&self, view: usize) -> Result<Option<Arc<Pose>>> {
        Ok(self
            .0
            .poses
            .lock()
            .map_err(|_| anyhow::anyhow!("GPU batch pose lock"))?
            .get(&view)
            .cloned())
    }
}

impl Pose {
    fn fail(&self, reason: impl Into<String>) {
        self.invalid.store(true, Ordering::Release);
        if let Ok(mut failure) = self.failure.lock() {
            failure.get_or_insert_with(|| reason.into());
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "native extraction joins one checked fixture, tags and its real camera"
)]
fn extract(
    mut frame: ResMut<Frame>,
    observer: Res<OrdinaryBatches>,
    state: Extract<Option<Res<SceneCaptureState>>>,
    mark: Extract<Res<observation::VisibilityMark>>,
    inventory: Extract<Res<inventory::CurrentInventory>>,
    subject: Extract<Res<Subject>>,
    tags: Extract<Query<&MeshTag>>,
    cameras: Extract<Query<(Entity, &Camera), With<crate::presentation::TacticalGameplayCamera>>>,
) -> bevy::prelude::Result {
    frame.pose = None;
    frame.serial = frame.serial.next()?;
    frame
        .commands
        .lock()
        .map_err(|_| "GPU command lock")?
        .clear();
    let Some(state) = state.as_ref() else {
        return Ok(());
    };
    if state.phase == CapturePhase::Configure || state.view >= state.views.len() {
        return Ok(());
    }
    if mark.0 != Some(state.view) || !inventory.valid {
        if matches!(
            **subject,
            Subject::Generated {
                selection: LevelSelection::Adaptive,
                ..
            }
        ) && observer.0.control == BatchControl::Ordinary
        {
            return Err("adaptive extraction lacks this frame's camera/selection inventory".into());
        }
        observer.invalidate(state.view);
        return Ok(());
    }
    let (camera, active) = cameras.single()?;
    if !active.is_active {
        return Err("GPU proof camera inactive".into());
    }
    if let Some(selection) = &inventory.selection
        && (selection.view != state.view || selection.camera.entity != camera)
    {
        return Err("adaptive extracted camera/checkpoint owner changed".into());
    }
    observer.check_retained_tags(|entity| tags.get(entity).ok().cloned())?;
    let saved = observer.0.tags.lock().map_err(|_| "GPU tag lock")?;
    let expected = TaggedMember::collect(&inventory, &saved)?;
    if expected.is_empty() {
        return Err("empty GPU proof inventory".into());
    }
    for member in &expected {
        if tags.get(member.entity)?.0 != member.tag.0 {
            return Err("extracted mesh tag drift".into());
        }
    }
    let retained = RetainedViewEntity::new(camera.into(), None, 0);
    let mut poses = observer.0.poses.lock().map_err(|_| "GPU pose lock")?;
    let pose = poses.entry(state.view).or_insert_with(|| {
        Arc::new(Pose {
            view: state.view,
            subject: **subject,
            first_frame: frame.serial,
            readiness: Readiness::for_view(**subject, observer.0.control, &state.views[state.view]),
            selection: inventory.selection.clone(),
            buffers: inventory.buffers.clone(),
            camera: retained,
            camera_bits: camera.to_bits(),
            expected: expected.clone(),
            output: state.output.clone(),
            invalid: AtomicBool::new(false),
            complete: AtomicBool::new(false),
            failure: Mutex::new(None),
            submission: Mutex::new(None),
        })
    });
    if pose.camera != retained
        || pose.expected != expected
        || pose.subject != **subject
        || pose.buffers != inventory.buffers
        || pose.selection != inventory.selection
    {
        pose.fail("held GPU pose membership or camera changed");
    }
    frame.pose = Some(Arc::clone(pose));
    Ok(())
}

fn apply_control(world: &mut World) -> bevy::prelude::Result {
    if matches!(world.resource::<Subject>(), Subject::Generated { .. }) {
        return Ok(());
    }
    let observer = world.resource::<OrdinaryBatches>();
    if world.resource::<SceneCaptureState>().view == 0 {
        return Ok(());
    }
    match observer.0.control {
        BatchControl::HiddenLastRoot => {
            *world.resource_mut::<controls::RootVisibility>() =
                controls::RootVisibility::LastRootHidden;
            let hidden = world
                .resource::<super::inventory::OwnedGrown>()
                .last_root()?;
            if world.get::<Visibility>(hidden) != Some(&Visibility::Hidden) {
                world.entity_mut(hidden).insert(Visibility::Hidden);
            }
        }
        BatchControl::MissingLastTag => {
            let part = world
                .resource::<super::inventory::OwnedGrown>()
                .last_part()?;
            world.entity_mut(part).remove::<MeshTag>();
        }
        BatchControl::ChangedLevel => {
            let held = world.resource::<super::inventory::OwnedGrown>().level();
            world.resource_mut::<Review>().level = match held {
                Level::Full => Level::Middle,
                Level::Middle | Level::Coarse => Level::Full,
            };
        }
        BatchControl::MissingInactiveTag
        | BatchControl::MissingVistaTag
        | BatchControl::HiddenVistaRoot => {
            return Err("selected control requires generated tree roots".into());
        }
        BatchControl::Ordinary
        | BatchControl::ForcedSingletons
        | BatchControl::ChangedSceneOwner
        | BatchControl::ParentedOriginalRoot => {}
    }
    Ok(())
}
