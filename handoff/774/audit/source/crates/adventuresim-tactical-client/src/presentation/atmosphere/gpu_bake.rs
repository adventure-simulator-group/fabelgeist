//! A bake finishes after actual atmosphere and filtering submissions, never asset allocation.
use super::*;
use bevy::{
    core_pipeline::schedule::{Core3d, Core3dSystems},
    ecs::system::SystemParam,
    pbr::{
        GpuScatteringMedium, LightMeta, ViewLightsUniformOffset,
        generate::{GeneratorBindGroups, GeneratorPipelines, RenderEnvironmentMap},
        resources::{
            AtmosphereTextures, AtmosphereTransforms, AtmosphereTransformsOffset, GpuAtmosphere,
        },
    },
    render::{
        Render, RenderSystems,
        extract_component::ComponentUniforms,
        render_asset::RenderAssets,
        render_resource::{CachedPipelineState, PipelineCache, PipelineDescriptor},
        renderer::{RenderQueue, ViewQuery},
        sync_world::MainEntity,
        texture::GpuImage,
        view::{ViewDepthTexture, ViewUniformOffset, ViewUniforms},
    },
    shader::Shader,
};
use std::sync::{Arc, Mutex};

// Bevy 0.19 keeps atmosphere pipeline IDs private. Match their canonical shader assets,
// not diagnostic pipeline labels; all five compute stages must exist and be compiled.
const ATMOSPHERE_COMPUTE_SHADERS: [&str; 5] = [
    "embedded://bevy_pbr/atmosphere/transmittance_lut.wgsl",
    "embedded://bevy_pbr/atmosphere/multiscattering_lut.wgsl",
    "embedded://bevy_pbr/atmosphere/sky_view_lut.wgsl",
    "embedded://bevy_pbr/atmosphere/aerial_view_lut.wgsl",
    "embedded://bevy_pbr/atmosphere/environment.wgsl",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BakeRequest {
    scene: Entity,
    camera: Entity,
    probe: Entity,
    images: [AssetId<Image>; 3],
}

#[derive(Default, Debug, PartialEq, Eq)]
enum BakeStage {
    #[default]
    Waiting,
    SourceRendered,
    FilteringRendered,
    AwaitingGpu,
    Complete,
}

#[derive(Default)]
struct BakeProgress {
    request: Option<BakeRequest>,
    stage: BakeStage,
}

#[derive(Resource, Clone, Default)]
pub(in crate::presentation) struct AtmosphereBakeGpu {
    progress: Arc<Mutex<BakeProgress>>,
    shaders: [Handle<Shader>; 5],
}

impl AtmosphereBakeGpu {
    pub(super) fn install(app: &mut App) {
        app.init_resource::<Self>();
        if app.get_sub_app(RenderApp).is_none() {
            return;
        }
        let asset_server = app.world().resource::<AssetServer>();
        let readiness = Self {
            shaders: ATMOSPHERE_COMPUTE_SHADERS.map(|path| asset_server.load(path)),
            ..default()
        };
        app.insert_resource(readiness.clone());
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .insert_resource(readiness)
                .add_systems(Core3d, observe_rendered_bake.after(Core3dSystems::MainPass))
                .add_systems(Render, fence_completed_bake.in_set(RenderSystems::Cleanup));
        }
    }

    pub(super) fn is_complete(
        &self,
        scene: Entity,
        camera: Entity,
        probe: Entity,
        generated: &GeneratedEnvironmentMapLight,
        filtered: &EnvironmentMapLight,
    ) -> bool {
        let request = BakeRequest {
            scene,
            camera,
            probe,
            images: [
                generated.environment_map.id(),
                filtered.diffuse_map.id(),
                filtered.specular_map.id(),
            ],
        };
        let mut progress = self.progress.lock().expect("atmosphere bake progress");
        if progress.request != Some(request) {
            *progress = BakeProgress {
                request: Some(request),
                ..default()
            };
        }
        progress.stage == BakeStage::Complete
    }

    #[cfg(test)]
    pub(super) fn complete_for_test(&self) {
        self.progress.lock().unwrap().stage = BakeStage::Complete;
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the rendered view must carry the actual atmosphere uniforms and LUT textures"
)]
fn observe_rendered_bake(
    view: ViewQuery<(
        &MainEntity,
        Option<(
            &AtmosphereTextures,
            &DynamicUniformIndex<GpuAtmosphere>,
            &DynamicUniformIndex<GpuAtmosphereSettings>,
            &ExtractedAtmosphere,
            &AtmosphereTransformsOffset,
            &ViewUniformOffset,
            &ViewLightsUniformOffset,
            &ViewDepthTexture,
            &Msaa,
        )>,
    )>,
    readiness: Res<AtmosphereBakeGpu>,
    pipelines: Res<PipelineCache>,
    filtering: Option<Res<GeneratorPipelines>>,
    images: Res<RenderAssets<GpuImage>>,
    bindings: PreparedAtmosphereBindings,
    probes: Query<(&MainEntity, &RenderEnvironmentMap, &GeneratorBindGroups)>,
) {
    let (camera, atmosphere_view) = view.into_inner();
    let mut progress = readiness.progress.lock().expect("atmosphere bake progress");
    let Some(request) = progress.request else {
        return;
    };
    if camera.id() != request.camera {
        return;
    }
    let Some((_, _, _, atmosphere, ..)) = atmosphere_view else {
        progress.stage = BakeStage::Waiting;
        return;
    };
    // Bind-group preparation requires a resident scattering medium as well as
    // these prepared view/light/atmosphere offsets and depth/LUT attachments.
    let ready = compute_ready(&readiness, &pipelines, filtering.as_deref())
        && bindings.is_ready(atmosphere)
        && probes.iter().any(|(entity, maps, _)| {
            entity.id() == request.probe
                && [
                    maps.environment_map.texture.id(),
                    maps.diffuse_map.texture.id(),
                    maps.specular_map.texture.id(),
                ]
                .into_iter()
                .zip(request.images)
                .all(|(texture, asset)| {
                    images
                        .get(asset)
                        .is_some_and(|image| image.texture.id() == texture)
                })
        });
    if !ready {
        progress.stage = BakeStage::Waiting;
        return;
    }
    // Bevy filters environment maps before Core3d generates this frame's atmosphere.
    // The next rendered frame is therefore the first that can filter the completed cube.
    progress.stage = match progress.stage {
        BakeStage::Waiting => BakeStage::SourceRendered,
        BakeStage::SourceRendered => BakeStage::FilteringRendered,
        _ => return,
    };
}

/// Public prerequisites checked by Bevy's private atmosphere bind-group preparation.
#[derive(SystemParam)]
struct PreparedAtmosphereBindings<'w> {
    media: Res<'w, RenderAssets<GpuScatteringMedium>>,
    atmosphere: Res<'w, ComponentUniforms<GpuAtmosphere>>,
    settings: Res<'w, ComponentUniforms<GpuAtmosphereSettings>>,
    transforms: Option<Res<'w, AtmosphereTransforms>>,
    views: Res<'w, ViewUniforms>,
    lights: Res<'w, LightMeta>,
}

impl PreparedAtmosphereBindings<'_> {
    fn is_ready(&self, atmosphere: &ExtractedAtmosphere) -> bool {
        self.media.get(atmosphere.medium).is_some()
            && self.atmosphere.binding().is_some()
            && self.settings.binding().is_some()
            && self
                .transforms
                .as_ref()
                .is_some_and(|transforms| transforms.uniforms().binding().is_some())
            && self.views.uniforms.binding().is_some()
            && self.lights.view_gpu_lights.binding().is_some()
    }
}

fn compute_ready(
    readiness: &AtmosphereBakeGpu,
    cache: &PipelineCache,
    filtering: Option<&GeneratorPipelines>,
) -> bool {
    let Some(filtering) = filtering else {
        return false;
    };
    let filter_ready = [
        filtering.copy,
        filtering.downsample_first,
        filtering.downsample_second,
        filtering.radiance,
        filtering.irradiance,
    ]
    .into_iter()
    .all(|pipeline| cache.get_compute_pipeline(pipeline).is_some());
    filter_ready
        && readiness.shaders.iter().all(|shader| {
            cache.pipelines().any(|pipeline| {
                let PipelineDescriptor::ComputePipelineDescriptor(descriptor) =
                    &pipeline.descriptor
                else {
                    return false;
                };
                descriptor.shader.id() == shader.id()
                    && descriptor.shader_defs.is_empty()
                    && matches!(pipeline.state, CachedPipelineState::Ok(_))
            })
        })
}

fn fence_completed_bake(readiness: Res<AtmosphereBakeGpu>, queue: Res<RenderQueue>) {
    let mut progress = readiness.progress.lock().expect("atmosphere bake progress");
    if progress.stage != BakeStage::FilteringRendered {
        return;
    }
    let request = progress.request;
    progress.stage = BakeStage::AwaitingGpu;
    let shared = readiness.progress.clone();
    drop(progress);
    // Cleanup follows render_system's queue submissions. This callback proves both
    // atmosphere generation and next-frame filtering have finished on the GPU.
    queue.on_submitted_work_done(move || {
        let mut progress = shared.lock().expect("atmosphere bake progress");
        if progress.request == request && progress.stage == BakeStage::AwaitingGpu {
            progress.stage = BakeStage::Complete;
        }
    });
}
