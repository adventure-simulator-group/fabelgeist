//! Successful actual production material commands, held through screenshot callbacks.
//! A queue prefix is a prerequisite, not an exact screenshot-frame timestamp.
use super::super::capture_state::{CapturePhase, SceneCaptureState};
use super::*;
use crate::presentation::TacticalGameplayCamera;
use bevy::{
    camera::visibility::{VisibilitySystems, VisibleEntities},
    core_pipeline::core_3d::{AlphaMask3d, Opaque3d},
    ecs::system::{SystemParamItem, SystemState, lifetimeless::SRes},
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
use inventory::Member;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Resource, Clone, Default)]
pub(super) struct SpecimenDraws(Arc<Mutex<BTreeMap<usize, Arc<Pose>>>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SingleDrawPath {
    Direct,
    DynamicUniformOffset { offset: u32 },
    Indirect { parameter: u32 },
}

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
    drawn: Mutex<HashMap<Entity, BindingReceipt>>,
    bindings: Mutex<Option<HashMap<Entity, BindingReceipt>>>,
    invalid: AtomicBool,
    queued: AtomicBool,
    complete: AtomicBool,
}

#[derive(Resource, Default)]
struct Frame(Option<Arc<Pose>>);

#[derive(Resource, Default)]
struct VisibilityMark(Option<usize>);

struct Record;

impl SpecimenDraws {
    pub(super) fn install(app: &mut App) -> Self {
        let draws = Self::default();
        app.insert_resource(draws.clone())
            .init_resource::<VisibilityMark>()
            .init_resource::<inventory::CurrentInventory>()
            .add_systems(
                PostUpdate,
                observe.after(VisibilitySystems::CheckVisibility).after(
                    crate::presentation::interior_lighting::InteriorLightingSystems::Prepare,
                ),
            );
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
            "queue_prefix_complete": true,
            "scope": "actual successful single-member DrawMaterial commands; per-frame held mesh/material bindings checked through callback; not exact screenshot GPU timestamps"}))
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

fn observe(world: &mut World) -> bevy::prelude::Result {
    world.resource_mut::<VisibilityMark>().0 = None;
    let state = world.resource::<SceneCaptureState>();
    if state.phase == CapturePhase::Configure || state.view >= state.views.len() {
        return Ok(());
    }
    let view = state.view;
    let CapturePose::Cache(expected) = state.views[view].pose else {
        return Ok(());
    };
    let (camera, transform, projection) = world
        .query_filtered::<(&Camera, &GlobalTransform, &Projection), With<TacticalGameplayCamera>>()
        .single(world)?;
    if !expected.matches(
        camera,
        transform,
        projection,
        world.resource::<cameras::PerspectiveDepth>(),
    ) {
        world.resource::<Evidence>().camera_drift(view)?;
        world.resource::<SpecimenDraws>().invalidate(view);
        return Ok(());
    }
    inventory::OwnedSpecimen::instrument_children(world)?;
    let checked = {
        let mut inspection = SystemState::<inventory::Inspector>::new(world);
        inspection.get(world).map_err(anyhow::Error::from)?.check()
    };
    match checked {
        Ok(inventory) => {
            world.insert_resource(inventory);
            world.resource_mut::<VisibilityMark>().0 = Some(view);
        }
        Err(error) => {
            world
                .resource::<Evidence>()
                .input_fault(view, format!("{error:#}"))?;
            world.resource::<SpecimenDraws>().invalidate(view);
            return Err(error.into());
        }
    }
    Ok(())
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
    if !expected.iter().any(|member| member.part.is_wood())
        || (camera_pose.plate != cameras::Plate::Root
            && !expected.iter().any(|member| member.part.is_foliage()))
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
            drawn: Mutex::new(HashMap::new()),
            bindings: Mutex::new(None),
            invalid: AtomicBool::new(false),
            queued: AtomicBool::new(false),
            complete: AtomicBool::new(false),
        })
    });
    if pose.camera != camera_id || pose.expected != expected {
        pose.invalid.store(true, Ordering::Release);
    }
    if let Ok(mut drawn) = pose.drawn.lock() {
        drawn.clear();
    } else {
        pose.invalid.store(true, Ordering::Release);
    }
    frame.0 = Some(Arc::clone(pose));
}

impl<P: PhaseItem> RenderCommand<P> for Record {
    type Param = (
        SRes<Frame>,
        SRes<RenderMeshInstances>,
        SRes<RenderMaterialInstances>,
        SRes<ErasedRenderAssets<PreparedMaterial>>,
    );
    type ViewQuery = &'static ExtractedView;
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        view: &'w ExtractedView,
        _entity: Option<()>,
        params: SystemParamItem<'w, '_, Self::Param>,
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (frame, meshes, materials, prepared) = params;
        let Some(pose) = &frame.into_inner().0 else {
            return RenderCommandResult::Success;
        };
        if pose.camera != view.retained_view_entity {
            return RenderCommandResult::Success;
        }
        let Some(member) = pose
            .expected
            .iter()
            .find(|member| member.entity == item.main_entity().id())
        else {
            return RenderCommandResult::Success;
        };
        let mesh = meshes.into_inner().mesh_asset_id(item.main_entity());
        let material = materials
            .into_inner()
            .instances
            .get(&item.main_entity())
            .map(|instance| instance.asset_id);
        let Some(actual) = material.and_then(|id| prepared.into_inner().get(id)) else {
            pose.invalid.store(true, Ordering::Release);
            return RenderCommandResult::Success;
        };
        if mesh != Some(member.mesh)
            || material != Some(member.material)
            || item.batch_range().len() != 1
        {
            pose.invalid.store(true, Ordering::Release);
            return RenderCommandResult::Success;
        }
        let path = match item.extra_index() {
            PhaseItemExtraIndex::None => SingleDrawPath::Direct,
            PhaseItemExtraIndex::DynamicOffset(offset) => {
                SingleDrawPath::DynamicUniformOffset { offset }
            }
            PhaseItemExtraIndex::IndirectParametersIndex {
                range,
                batch_set_index: None,
            } if range.len() == 1 => SingleDrawPath::Indirect {
                parameter: range.start,
            },
            _ => {
                pose.invalid.store(true, Ordering::Release);
                return RenderCommandResult::Success;
            }
        };
        let receipt = BindingReceipt {
            mesh: member.mesh,
            material: member.material,
            group: actual.binding.group.0,
            slot: actual.binding.slot.0,
            path,
        };
        if let Ok(bindings) = pose.bindings.lock() {
            if bindings.as_ref().is_some_and(|bindings| {
                !bindings
                    .get(&member.entity)
                    .is_some_and(|binding| binding.same_binding(&receipt))
            }) {
                pose.invalid.store(true, Ordering::Release);
            }
        } else {
            pose.invalid.store(true, Ordering::Release);
        }
        if let Ok(mut drawn) = pose.drawn.lock() {
            drawn.insert(member.entity, receipt);
        } else {
            pose.invalid.store(true, Ordering::Release);
        }
        RenderCommandResult::Success
    }
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
