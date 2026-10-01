use super::*;
use bevy::{
    camera::primitives::Frustum,
    core_pipeline::schedule::{Core3d, Core3dSystems, RootNonCameraView},
    pbr::{
        EARLY_SHADOW_PASS, ShadowView, ViewLightEntities, per_view_shadow_pass, shared_shadow_pass,
    },
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        render_resource::{binding_types::*, *},
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        storage::GpuShaderBuffer,
        view::{ExtractedView, RetainedViewEntity},
    },
};
use std::collections::{HashMap, HashSet};

const CULL_SHADER: &str = "shaders/tactical_city_cull.wgsl";
const COMPUTE_WORKGROUP_SIZE: u32 = 64;
// WebGPU guarantees this many workgroups per dimension. Large material batches
// span rows without depending on stronger device limits.
const WORKGROUPS_PER_ROW: u32 = 65_535;
const FACADE_RADIUS_PIXELS: f32 = 48.0;
const DETAIL_RADIUS_PIXELS: f32 = 180.0;

#[derive(Clone, ShaderType)]
struct ViewParameters {
    clip_from_world: Mat4,
    camera: Vec4,
    projection: Vec4,
    counts: UVec4,
    policy: UVec4,
    far_plane: Vec4,
}

#[derive(Resource)]
struct Pipelines {
    layout: BindGroupLayoutDescriptor,
    select: CachedComputePipelineId,
    compact: CachedComputePipelineId,
}

struct Dispatch {
    binding: BindGroup,
    view_offset: u32,
    ranges: u32,
    slot: u32,
}

#[derive(bevy::ecs::query::QueryData)]
struct CityView {
    entity: Entity,
    view: &'static ExtractedView,
    camera: Has<ExtractedCamera>,
    cascades: Option<&'static ViewLightEntities>,
    root: Has<RootNonCameraView>,
    shadow: Has<ShadowView>,
    frustum: Option<&'static Frustum>,
}

fn camera_groups(views: &Query<CityView, Without<Camera2d>>) -> Vec<(Entity, Vec<Entity>)> {
    views
        .iter()
        .filter(|view| view.camera || view.root)
        .map(|view| {
            let mut group = vec![view.entity];
            if let Some(cascades) = view.cascades {
                group.extend(cascades.lights.iter().copied());
            }
            group.retain(|entity| views.get(*entity).is_ok());
            (view.entity, group)
        })
        .collect()
}

#[derive(Resource, Default)]
pub(super) struct CityViews {
    pub slots: HashMap<Entity, u32>,
    uniforms: DynamicUniformBuffer<ViewParameters>,
    dispatches: HashMap<Entity, Vec<Dispatch>>,
    retained_slots: HashMap<RetainedViewEntity, u32>,
}

impl CityViews {
    fn prepare_views(
        &mut self,
        scene: &CityGpuScene,
        groups: &[(Entity, Vec<Entity>)],
        views: &Query<CityView, Without<Camera2d>>,
    ) -> Vec<(Entity, u32, u32)> {
        let active: HashSet<_> = groups
            .iter()
            .flat_map(|(_, group)| group.iter())
            .filter_map(|entity| views.get(*entity).ok().map(|v| v.view.retained_view_entity))
            .collect();
        assert!(
            active.len() <= MAX_CITY_VIEWS,
            "city LOD history capacity exceeded"
        );
        self.retained_slots.retain(|key, _| active.contains(key));
        let mut offsets = Vec::new();
        for (root, group) in groups {
            for (draw_slot, entity) in group.iter().enumerate() {
                let item = views.get(*entity).expect("active city view");
                let view = item.view;
                let previous = self.retained_slots.get(&view.retained_view_entity).copied();
                let history_slot = previous.unwrap_or_else(|| {
                    (0..MAX_CITY_VIEWS as u32)
                        .find(|slot| !self.retained_slots.values().any(|used| used == slot))
                        .expect("bounded city LOD history")
                });
                self.retained_slots
                    .insert(view.retained_view_entity, history_slot);
                self.slots.insert(*entity, draw_slot as u32);
                let clip = view.clip_from_world.unwrap_or_else(|| {
                    view.clip_from_view * view.world_from_view.to_matrix().inverse()
                });
                for batch in scene.batches.iter() {
                    let offset = self.uniforms.push(&ViewParameters {
                        clip_from_world: clip,
                        camera: views
                            .get(*root)
                            .expect("camera group root")
                            .view
                            .world_from_view
                            .translation()
                            .extend(view.clip_from_view.w_axis.w),
                        projection: Vec4::new(
                            view.clip_from_view.y_axis.y.abs() * view.viewport.w as f32 * 0.5,
                            FACADE_RADIUS_PIXELS,
                            DETAIL_RADIUS_PIXELS,
                            if previous.is_some() { 1.0 } else { 0.0 },
                        ),
                        counts: UVec4::new(
                            scene.count,
                            batch.capacity,
                            history_slot,
                            draw_slot as u32,
                        ),
                        // A shadow map needs the enclosure silhouette, not
                        // the facade's individual beams and window fittings.
                        // Test view role explicitly: orthographic cameras are
                        // not shadow views.
                        policy: UVec4::new(
                            u32::from(item.shadow),
                            u32::from(views.get(*root).expect("camera group root").root),
                            0,
                            0,
                        ),
                        // Perspective matrices have an infinite far plane.
                        // Bevy's camera frustum retains the configured finite
                        // distance; shadow caster extrusion must remain separate.
                        far_plane: if item.shadow {
                            Vec4::ZERO
                        } else {
                            item.frustum
                                .map_or(Vec4::ZERO, |frustum| frustum.half_spaces[5].normal_d())
                        },
                    });
                    offsets.push((*root, offset, draw_slot as u32));
                }
            }
        }
        offsets
    }
}

pub(super) fn install(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<CityViews>()
        .init_resource::<scratch::Scratch>()
        .add_systems(RenderStartup, initialize)
        .add_systems(Render, prepare.in_set(RenderSystems::PrepareBindGroups))
        .add_systems(
            Core3d,
            cull.before(Core3dSystems::Prepass)
                .before(per_view_shadow_pass::<EARLY_SHADOW_PASS>)
                .before(shared_shadow_pass::<EARLY_SHADOW_PASS>),
        );
}

fn initialize(mut commands: Commands, cache: Res<PipelineCache>, server: Res<AssetServer>) {
    let layout = BindGroupLayoutDescriptor::new(
        "city GPU culling",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<ViewParameters>(true),
                storage_buffer_read_only::<assembly::Building>(false),
                storage_buffer::<u32>(false),
                storage_buffer_read_only::<UVec4>(false),
                storage_buffer::<UVec2>(false),
                storage_buffer::<UVec4>(false),
            ),
        ),
    );
    let shader = server.load(CULL_SHADER);
    let pipeline = |entry: &str| {
        cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(format!("city_{entry}").into()),
            layout: vec![layout.clone()],
            shader: shader.clone(),
            entry_point: Some(entry.to_owned().into()),
            ..default()
        })
    };
    let select = pipeline("select_buildings");
    let compact = pipeline("compact_ranges");
    commands.insert_resource(Pipelines {
        layout,
        select,
        compact,
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "GPU preparation joins resident assets, active views and device resources"
)]
fn prepare(
    scene: Res<CityGpuScene>,
    pipelines: Res<Pipelines>,
    cache: Res<PipelineCache>,
    buffers: Res<RenderAssets<GpuShaderBuffer>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    views: Query<CityView, Without<Camera2d>>,
    mut prepared: ResMut<CityViews>,
    mut scratch: ResMut<scratch::Scratch>,
) {
    prepared.slots.clear();
    prepared.dispatches.clear();
    prepared.uniforms.clear();
    let groups = camera_groups(&views);
    let slots = groups
        .iter()
        .map(|(_, group)| group.len())
        .max()
        .unwrap_or(1);
    if !scratch.prepare(&scene, slots, &buffers, &device, &queue, &cache) {
        return;
    }
    let (Some(buildings), Some(selection)) =
        (buffers.get(&scene.buildings), buffers.get(&scene.selection))
    else {
        return;
    };
    let offsets = prepared.prepare_views(&scene, &groups, &views);
    prepared.uniforms.write_buffer(&device, &queue);
    let Some(uniform) = prepared.uniforms.binding() else {
        return;
    };
    let layout = cache.get_bind_group_layout(&pipelines.layout);
    let mut bindings = Vec::new();
    for (batch, scratch) in scene.batches.iter().zip(&scratch.batches) {
        let Some(source) = buffers.get(&batch.source) else {
            return;
        };
        bindings.push(device.create_bind_group(
            "city GPU buffers",
            &layout,
            &BindGroupEntries::sequential((
                uniform.clone(),
                buildings.buffer.as_entire_binding(),
                selection.buffer.as_entire_binding(),
                source.buffer.as_entire_binding(),
                scratch.visible.as_entire_binding(),
                scratch.indirect.as_entire_binding(),
            )),
        ));
    }
    for (index, (root, offset, slot)) in offsets.into_iter().enumerate() {
        let batch = index % bindings.len();
        prepared.dispatches.entry(root).or_default().push(Dispatch {
            binding: bindings[batch].clone(),
            view_offset: offset,
            ranges: scene.batches[batch].ranges,
            slot,
        });
    }
}

fn cull(
    mut context: RenderContext,
    pipelines: Res<Pipelines>,
    cache: Res<PipelineCache>,
    scene: Res<CityGpuScene>,
    views: Res<CityViews>,
    current: ViewQuery<Entity>,
    scratch: Res<scratch::Scratch>,
) {
    let Some(dispatches) = views.dispatches.get(&current.into_inner()) else {
        return;
    };
    if dispatches.is_empty() {
        return;
    }
    let (Some(select), Some(compact)) = (
        cache.get_compute_pipeline(pipelines.select),
        cache.get_compute_pipeline(pipelines.compact),
    ) else {
        return;
    };
    // Camera groups execute sequentially. Clear only counters here, in GPU
    // command order; queue writes would reset all groups before any draws run.
    for (index, dispatch) in dispatches.iter().enumerate() {
        let batch = &scratch.batches[index % scene.batches.len()];
        context.command_encoder().clear_buffer(
            &batch.indirect,
            u64::from(dispatch.slot) * 16 + 4,
            Some(4),
        );
    }
    {
        let mut pass = context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("city_building_lod"),
                timestamp_writes: None,
            });
        pass.set_pipeline(select);
        for dispatch in dispatches.iter().step_by(scene.batches.len()) {
            pass.set_bind_group(0, &dispatch.binding, &[dispatch.view_offset]);
            pass.dispatch_workgroups(scene.count.div_ceil(COMPUTE_WORKGROUP_SIZE), 1, 1);
        }
    }
    let mut pass = context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("city_visible_clusters"),
            timestamp_writes: None,
        });
    pass.set_pipeline(compact);
    for dispatch in dispatches {
        pass.set_bind_group(0, &dispatch.binding, &[dispatch.view_offset]);
        let workgroups = dispatch.ranges.div_ceil(COMPUTE_WORKGROUP_SIZE);
        pass.dispatch_workgroups(
            workgroups.min(WORKGROUPS_PER_ROW),
            workgroups.div_ceil(WORKGROUPS_PER_ROW),
            1,
        );
    }
    READY.store(true, Ordering::Relaxed);
}
