//! The studio's hair GPU: the hair kernels on the device the studio draws
//! with, so hair grown there can be drawn from where it lies. Compiling the
//! kernels takes a moment, so the first worker that grows hair compiles
//! them, and every later one reuses them.
use super::*;
use adventuresim_character_creator::hair::HairGpu;
use bevy::render::RenderApp;
use bevy::render::renderer::{RenderAdapter, RenderDevice, RenderInstance, RenderQueue};
use fabelgeist_gpu::globals::WgpuContext;
use std::sync::OnceLock;

/// The studio's render device, and the hair kernels on it once compiled.
#[derive(Clone)]
pub(crate) struct StudioHairGpu(Arc<DeviceKernels>);

struct DeviceKernels {
    context: WgpuContext,
    kernels: OnceLock<Result<HairGpu, String>>,
}

impl StudioHairGpu {
    /// The hair kernels, compiled on the first call.
    pub(crate) fn kernels(&self) -> Result<&HairGpu> {
        self.0
            .kernels
            .get_or_init(|| {
                HairGpu::new(self.0.context.clone()).map_err(|error| format!("{error:#}"))
            })
            .as_ref()
            .map_err(|error| anyhow::anyhow!("compiling the hair kernels: {error}"))
    }
}

/// Hands the hair preview the studio's render device once the renderer has
/// one.
pub(crate) struct HairGpuPlugin;

impl Plugin for HairGpuPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let Some(instance) = app
            .get_sub_app(RenderApp)
            .and_then(|render| render.world().get_resource::<RenderInstance>())
            .map(|instance| (**instance.0).clone())
        else {
            return;
        };
        let world = app.world();
        let (Some(device), Some(queue), Some(adapter)) = (
            world.get_resource::<RenderDevice>(),
            world.get_resource::<RenderQueue>(),
            world.get_resource::<RenderAdapter>(),
        ) else {
            return;
        };
        let context = WgpuContext::from_parts(
            Arc::new(instance),
            Arc::new((**adapter.0).clone()),
            Arc::new(device.wgpu_device().clone()),
            Arc::new((**queue.0).clone()),
        );
        let gpu = StudioHairGpu(Arc::new(DeviceKernels {
            context,
            kernels: OnceLock::new(),
        }));
        if let Some(mut preview) = app.world_mut().get_resource_mut::<HairPreview>() {
            preview.gpu = Some(gpu);
        }
    }
}
