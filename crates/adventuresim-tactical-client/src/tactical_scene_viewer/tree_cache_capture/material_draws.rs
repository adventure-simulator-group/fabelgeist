//! Successful actual production material commands, held through screenshot callbacks.
//! A queue prefix is a prerequisite, not an exact screenshot-frame timestamp.
use super::super::capture_state::{CapturePhase, SceneCaptureState};
use super::*;
use crate::presentation::TacticalGameplayCamera;
use bevy::{
    camera::visibility::VisibleEntities,
    core_pipeline::core_3d::{AlphaMask3d, Opaque3d},
    ecs::system::{SystemParamItem, lifetimeless::SRes},
    pbr::{DrawMaterial, PreparedMaterial, RenderMaterialInstances, RenderMeshInstances},
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        erased_render_asset::ErasedRenderAssets,
        render_phase::{
            DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand, RenderCommandResult,
            RenderCommandState, TrackedRenderPass,
        },
        renderer::RenderQueue,
        view::{ExtractedView, RetainedViewEntity},
    },
};
use command::Record;
use inventory::Member;
use observation::VisibilityMark;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

mod command;

#[derive(Resource, Clone, Default)]
pub(super) struct SpecimenDraws(Arc<Mutex<BTreeMap<usize, Arc<Pose>>>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BindingReceipt {
    mesh: AssetId<Mesh>,
    material: bevy::asset::UntypedAssetId,
    group: u32,
    slot: u32,
    path: SingleDrawPath,
}

struct Pose {
    view: usize,
    role: Role,
    camera: RetainedViewEntity,
    camera_bits: u64,
    expected: Vec<Member>,
    background: Vec<background::Member>,
    drawn: Mutex<HashMap<Entity, BindingReceipt>>,
    bindings: Mutex<Option<HashMap<Entity, BindingReceipt>>>,
    invalid: AtomicBool,
    queued: AtomicBool,
    complete: AtomicBool,
}

#[derive(Resource, Default)]
struct Frame(Option<Arc<Pose>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SingleDrawPath {
    Direct,
    DynamicUniformOffset { offset: u32 },
    Indirect { parameter: u32 },
}

impl SpecimenDraws {
    pub(super) fn install(app: &mut App) -> Self {
        let draws = Self::default();
        app.insert_resource(draws.clone());
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
        let bindings = pose.bindings.lock().ok()?;
        let bindings = bindings.as_ref()?;
        Some(serde_json::json!({"role": pose.role, "view": pose.view,
            "camera_entity_bits": pose.camera_bits,
            "expected": pose.expected.iter().map(|member| {
                let binding = bindings.get(&member.entity);
                serde_json::json!({"entity_bits": member.entity.to_bits(), "part": member.part,
                    "mesh": format!("{:?}", member.mesh), "effective_material": format!("{:?}", member.material),
                    "actual_material_group": binding.map(|binding| binding.group),
                    "actual_material_slot": binding.map(|binding| binding.slot), "single_draw_path": binding.map(|binding| binding.path)})
            }).collect::<Vec<_>>(),
            "terrain_background": pose.background.iter().map(|member| {
                let binding = bindings.get(&member.entity);
                serde_json::json!({"entity_bits": member.entity.to_bits(), "role": member.role,
                    "mesh": format!("{:?}", member.mesh), "world_from_mesh":member.transform.to_cols_array(),
                    "authored_material": format!("{:?}", member.material),
                    "effective_material": binding.map(|binding|format!("{:?}", binding.material)),
                    "actual_material_group":binding.map(|binding|binding.group), "actual_material_slot":binding.map(|binding|binding.slot)})
            }).collect::<Vec<_>>(),
            "terrain_background_complete": true,
            "queue_prefix_complete": true,
            "scope": "actual successful single-member DrawMaterial commands; per-frame held mesh/material bindings checked through callback; not exact screenshot GPU timestamps"}))
    }

    pub(super) fn check_background(
        &self,
        view: usize,
        current: &[background::Member],
    ) -> Result<()> {
        let poses = self
            .0
            .lock()
            .map_err(|_| anyhow::anyhow!("terrain draw proof lock"))?;
        let pose = poses.get(&view).context("held terrain draw pose")?;
        ensure!(
            !pose.invalid.load(Ordering::Acquire)
                && pose.complete.load(Ordering::Acquire)
                && pose.background == current,
            "screenshot callback terrain binding drift"
        );
        Ok(())
    }

    pub(super) fn invalidate(&self, view: usize) {
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

    pub(super) fn inventory_verified(&self) -> bool {
        self.0.lock().is_ok_and(|poses| {
            !poses.is_empty()
                && poses
                    .values()
                    .all(|pose| !pose.invalid.load(Ordering::Acquire))
        })
    }
}

#[expect(
    clippy::type_complexity,
    reason = "render extraction joins the selected inventory to the actual camera visibility"
)]
fn extract(
    mut frame: ResMut<Frame>,
    draws: Res<SpecimenDraws>,
    state: Extract<Option<Res<SceneCaptureState>>>,
    mark: Extract<Res<VisibilityMark>>,
    inventory: Extract<Res<inventory::CurrentInventory>>,
    review: Extract<Res<Review>>,
    cameras: Extract<Query<(Entity, &Camera, &VisibleEntities), With<TacticalGameplayCamera>>>,
) {
    frame.0 = None;
    let Some(state) = state.as_ref() else {
        return;
    };
    if state.phase == CapturePhase::Configure || state.view >= state.views.len() {
        return;
    }
    if mark.0 != Some(state.view) || !inventory.valid {
        draws.invalidate(state.view);
        return;
    }
    let Ok((camera, active, visible)) = cameras.single() else {
        draws.invalidate(state.view);
        return;
    };
    if !active.is_active {
        draws.invalidate(state.view);
        return;
    }
    let CapturePose::Cache(camera_pose) = state.views[state.view].pose else {
        return;
    };
    let camera_visible: HashSet<_> = visible
        .get(std::any::TypeId::of::<Mesh3d>())
        .iter()
        .copied()
        .collect();
    let expected: Vec<_> = inventory
        .members
        .iter()
        .filter(|member| {
            inventory.visible.contains(&member.entity) && camera_visible.contains(&member.entity)
        })
        .copied()
        .collect();
    if expected.is_empty()
        || (camera_pose.plate != cameras::Plate::Root
            && !expected.iter().any(|member| {
                matches!(
                    member.part,
                    crate::presentation::InstalledTreePartRole::CamberedLeaves { .. }
                        | crate::presentation::InstalledTreePartRole::FlatLeaves { .. }
                        | crate::presentation::InstalledTreePartRole::CanopyCard { .. }
                )
            }))
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
            role: review.role,
            camera: camera_id,
            camera_bits: camera.to_bits(),
            expected: expected.clone(),
            background: inventory.background.clone(),
            drawn: Mutex::new(HashMap::new()),
            bindings: Mutex::new(None),
            invalid: AtomicBool::new(false),
            queued: AtomicBool::new(false),
            complete: AtomicBool::new(false),
        })
    });
    if pose.camera != camera_id
        || pose.expected != expected
        || pose.background != inventory.background
    {
        pose.invalid.store(true, Ordering::Release);
    }
    if let Ok(mut drawn) = pose.drawn.lock() {
        drawn.clear();
    } else {
        pose.invalid.store(true, Ordering::Release);
    }
    frame.0 = Some(Arc::clone(pose));
}

fn complete(frame: Res<Frame>, queue: Res<RenderQueue>) {
    let Some(pose) = &frame.0 else {
        return;
    };
    if pose.invalid.load(Ordering::Acquire) {
        return;
    }
    let Ok(drawn) = pose.drawn.lock() else {
        pose.invalid.store(true, Ordering::Release);
        return;
    };
    if !pose
        .expected
        .iter()
        .all(|member| drawn.contains_key(&member.entity))
        || !pose
            .background
            .iter()
            .all(|member| drawn.contains_key(&member.entity))
    {
        if pose.queued.load(Ordering::Acquire) {
            pose.invalid.store(true, Ordering::Release);
        }
        return;
    }
    if pose.queued.load(Ordering::Acquire) {
        return;
    }
    if let Ok(mut bindings) = pose.bindings.lock() {
        *bindings = Some(drawn.clone());
    } else {
        pose.invalid.store(true, Ordering::Release);
        return;
    }
    pose.queued.store(true, Ordering::Release);
    let pose = Arc::clone(pose);
    queue.on_submitted_work_done(move || {
        pose.complete.store(true, Ordering::Release);
    });
}

impl BindingReceipt {
    fn same_binding(&self, other: &Self) -> bool {
        self.mesh == other.mesh
            && self.material == other.material
            && self.group == other.group
            && self.slot == other.slot
            && std::mem::discriminant(&self.path) == std::mem::discriminant(&other.path)
    }
}
