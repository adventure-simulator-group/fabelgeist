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
use draw_policy::BatchControl;
use members::TaggedMember;
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
mod members;
mod proof;
mod readbacks;
mod snapshots;
mod types;
mod validate;

const MAX_FIXTURE_MEMBERS: usize = 4096;

#[derive(Resource, Clone)]
pub(super) struct OrdinaryBatches(Arc<Shared>);

struct Shared {
    poses: Mutex<BTreeMap<usize, Arc<Pose>>>,
    tags: Mutex<HashMap<Entity, MeshTag>>,
    control: BatchControl,
}

struct Pose {
    view: usize,
    role: Role,
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

    pub(super) fn prepare_tags(
        &self,
        world: &mut World,
        inventory: &inventory::CurrentInventory,
    ) -> Result<()> {
        let mut saved = self
            .0
            .tags
            .lock()
            .map_err(|_| anyhow::anyhow!("tree tag lock"))?;
        for (entity, actual) in world.query::<(Entity, &MeshTag)>().iter(world) {
            ensure!(
                saved
                    .get(&entity)
                    .is_some_and(|expected| expected.0 == actual.0)
                    || (!saved.contains_key(&entity) && actual.0 == 0),
                "unexpected diagnostic mesh tag"
            );
        }
        let mut entities: Vec<_> = inventory
            .members
            .iter()
            .map(|member| member.entity)
            .chain(inventory.background.iter().map(|member| member.entity))
            .collect();
        entities.sort_unstable();
        entities.dedup();
        for entity in entities {
            if let Some(tag) = saved.get(&entity) {
                ensure!(
                    world
                        .get::<MeshTag>(entity)
                        .is_some_and(|actual| actual.0 == tag.0),
                    "owned tag disappeared or changed"
                );
                continue;
            }
            ensure!(saved.len() < MAX_FIXTURE_MEMBERS, "tree tag fixture budget");
            let tag = MeshTag(u32::try_from(
                saved.len().checked_add(1).context("tree tag overflow")?,
            )?);
            saved.insert(entity, tag.clone());
            world.entity_mut(entity).insert(tag);
        }
        Ok(())
    }

    pub(super) fn check_capture(
        &self,
        view: usize,
        current: &inventory::CurrentInventory,
        tags: &Query<&MeshTag>,
    ) -> Result<()> {
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
    review: Extract<Res<Review>>,
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
        observer.invalidate(state.view);
        return Ok(());
    }
    let (camera, active) = cameras.single()?;
    if !active.is_active {
        return Err("GPU proof camera inactive".into());
    }
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
            role: review.role,
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
    if pose.camera != retained || pose.expected != expected {
        pose.fail("held GPU pose membership or camera changed");
    }
    frame.pose = Some(Arc::clone(pose));
    Ok(())
}

fn apply_control(world: &mut World) -> bevy::prelude::Result {
    let observer = world.resource::<OrdinaryBatches>();
    if observer.0.control != BatchControl::HiddenLastRoot
        || world.resource::<SceneCaptureState>().view == 0
    {
        return Ok(());
    }
    *world.resource_mut::<draw_policy::RootVisibility>() =
        draw_policy::RootVisibility::LastRootHidden;
    let hidden = world.resource::<inventory::OwnedTrees>().last_root()?;
    if world.get::<Visibility>(hidden) != Some(&Visibility::Hidden) {
        world.entity_mut(hidden).insert(Visibility::Hidden);
    }
    Ok(())
}
