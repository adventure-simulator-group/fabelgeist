//! The studio's hair GPU: the hair kernels on the device the studio draws
//! with, so hair grown there can be drawn from where it lies. Compiling the
//! kernels takes a moment, so the first worker that grows hair compiles
//! them, and every later one reuses them.
use super::*;
use adventuresim_character_creator::GpuInitializationError;
use adventuresim_character_creator::hair::HairGpu;
use anyhow::Context;
use bevy::render::RenderApp;
use bevy::render::renderer::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue};
use fabelgeist_gpu::globals::WgpuContext;
use std::sync::OnceLock;

/// The studio's render device, and the hair kernels on it once compiled.
#[derive(Clone)]
pub(crate) struct StudioHairGpu(Arc<DeviceKernels>);

struct DeviceKernels {
    context: WgpuContext,
    kernels: OnceLock<Result<HairGpu, GpuInitializationError>>,
}

/// Hands the hair preview the studio's render device once the renderer has
/// one.
pub(crate) struct HairGpuPlugin;

impl StudioHairGpu {
    /// The first kernel initialization outcome on this device, including failure.
    pub(crate) fn kernels(&self) -> Result<&HairGpu> {
        self.0
            .kernels
            .get_or_init(|| {
                HairGpu::new(self.0.context.clone()).map_err(GpuInitializationError::new)
            })
            .as_ref()
            .map_err(Clone::clone)
            .context("compiling the hair kernels")
    }
}

impl Plugin for HairGpuPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let Some(context) = render_context(app) else {
            return;
        };
        let gpu = StudioHairGpu(Arc::new(DeviceKernels {
            context,
            kernels: OnceLock::new(),
        }));
        if let Some(mut preview) = app.world_mut().get_resource_mut::<HairPreview>() {
            preview.gpu = Some(gpu);
        }
    }
}

/// The device `app` renders with, once its renderer has one: hair built
/// there is drawn from where it lies.
pub(crate) fn render_context(app: &App) -> Option<WgpuContext> {
    let instance = app
        .get_sub_app(RenderApp)?
        .world()
        .get_resource::<RenderInstance>()
        .map(|instance| (**instance.0).clone())?;
    let world = app.world();
    let device = world.get_resource::<RenderDevice>()?;
    let queue = world.get_resource::<RenderQueue>()?;
    let adapter = world.get_resource::<RenderAdapter>()?;
    Some(WgpuContext::from_parts(
        Arc::new(instance),
        Arc::new((**adapter.0).clone()),
        Arc::new(device.wgpu_device().clone()),
        Arc::new((**queue.0).clone()),
    ))
}
