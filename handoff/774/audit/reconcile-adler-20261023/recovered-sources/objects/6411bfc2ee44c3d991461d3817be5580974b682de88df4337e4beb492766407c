//! Per-camera tuft culling. Grass has no prepass or shadow pass, so retained
//! output buffers can be reused immediately before each camera's main pass.
use std::collections::HashMap;

use bevy::{
    camera::primitives::Frustum,
    core_pipeline::schedule::{Core3d, Core3dSystems},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        view::ExtractedView,
    },
};
use bevy_eidolon::{
    allocator::resources::GlobalInstanceAllocator,
    cull::{pipeline::InstancedComputePipeline, resources::CameraCullData},
    prepass::CullComputeCamera,
};

use super::TacticalGrassInstancedMaterial;

const WORKGROUP_SIZE: u32 = 64;
const MAX_WORKGROUPS: u32 = 65_535;
type GrassPipeline = InstancedComputePipeline<TacticalGrassInstancedMaterial>;

pub(super) struct GrassCullingPlugin;

impl Plugin for GrassCullingPlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassViews>()
            .add_systems(Render, prepare.in_set(RenderSystems::PrepareBindGroups))
            .add_systems(
                Core3d,
                cull.after(Core3dSystems::Prepass)
                    .before(Core3dSystems::MainPass),
            );
    }

    fn finish(&self, app: &mut App) {
        let render = app.sub_app_mut(RenderApp);
        render.init_resource::<GrassPipeline>();
        let pipeline = render.world().resource::<GrassPipeline>();
        let cache = render.world().resource::<PipelineCache>();
        let queue = |entry: &str| {
            cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(format!("grass_{entry}").into()),
                layout: vec![
                    pipeline.compute_layout.clone(),
                    pipeline.common_layout.clone(),
                    pipeline.global_layout.clone(),
                ],
                shader: pipeline.shader.clone(),
                entry_point: Some(entry.to_owned().into()),
                ..default()
            })
        };
        let pipelines = Pipelines {
            reset: queue("reset"),
            select: queue("main"),
        };
        render.insert_resource(pipelines);
    }
}

#[derive(Resource)]
struct Pipelines {
    reset: CachedComputePipelineId,
    select: CachedComputePipelineId,
}

struct ViewBuffer {
    uniform: UniformBuffer<CameraCullData>,
    binding: BindGroup,
}

#[derive(Resource, Default)]
struct GrassViews(HashMap<Entity, ViewBuffer>);

fn prepare(
    views: Query<(Entity, &ExtractedView, &Frustum, &Camera), With<CullComputeCamera>>,
    mut prepared: ResMut<GrassViews>,
    pipeline: Res<GrassPipeline>,
    cache: Res<PipelineCache>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    prepared.0.retain(|entity, _| {
        views
            .get(*entity)
            .is_ok_and(|(_, _, _, camera)| camera.is_active)
    });
    for (entity, view, frustum, camera) in &views {
        if !camera.is_active {
            continue;
        }
        let data = CameraCullData {
            view_pos: view.world_from_view.translation().extend(1.0),
            frustum: frustum.half_spaces.map(|plane| plane.normal_d()),
        };
        if let Some(buffer) = prepared.0.get_mut(&entity) {
            buffer.uniform.set(data);
            buffer.uniform.write_buffer(&device, &queue);
        } else {
            let mut uniform = UniformBuffer::from(data);
            uniform.write_buffer(&device, &queue);
            let binding = device.create_bind_group(
                "grass culling camera",
                &cache.get_bind_group_layout(&pipeline.global_layout),
                &BindGroupEntries::single(uniform.binding().expect("uploaded camera")),
            );
            prepared.0.insert(entity, ViewBuffer { uniform, binding });
        }
    }
}

fn cull(
    mut context: RenderContext,
    current: ViewQuery<Entity>,
    views: Res<GrassViews>,
    pipelines: Res<Pipelines>,
    cache: Res<PipelineCache>,
    allocator: Res<GlobalInstanceAllocator<TacticalGrassInstancedMaterial>>,
) {
    let Some(view) = views.0.get(&current.into_inner()) else {
        return;
    };
    let (Some(reset), Some(select)) = (
        cache.get_compute_pipeline(pipelines.reset),
        cache.get_compute_pipeline(pipelines.select),
    ) else {
        return;
    };
    let mut pass = context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("grass_view_cull"),
            timestamp_writes: None,
        });
    pass.set_bind_group(2, &view.binding, &[]);
    for page in &allocator.pages {
        if page.compute_capacity == 0 {
            continue;
        }
        let (Some(compute), Some(common), Some(indirect)) = (
            &page.compute_bind_group,
            &page.common_bind_group,
            &page.indirect_buffer,
        ) else {
            continue;
        };
        pass.set_bind_group(0, compute, &[]);
        pass.set_bind_group(1, common, &[]);
        pass.set_pipeline(reset);
        let draws = indirect.size() / size_of::<DrawIndexedIndirectArgs>() as u64;
        pass.dispatch_workgroups((draws as u32).div_ceil(WORKGROUP_SIZE), 1, 1);
        pass.set_pipeline(select);
        let groups = page.compute_capacity.div_ceil(WORKGROUP_SIZE);
        pass.dispatch_workgroups(
            groups.min(MAX_WORKGROUPS),
            groups.div_ceil(MAX_WORKGROUPS),
            1,
        );
    }
}
