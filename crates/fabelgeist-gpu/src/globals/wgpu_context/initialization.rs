#[cfg(target_arch = "wasm32")]
use super::BrowserContextError;
use super::{Blitter, WgpuContext, report_adapter};
use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::Instance;

struct PresentationTarget {
    instance: Instance,
    surface: Option<Arc<wgpu::Surface<'static>>>,
    #[cfg(target_arch = "wasm32")]
    canvas: Option<web_sys::OffscreenCanvas>,
}

#[derive(Clone, Copy)]
enum ContextMode {
    Presentation,
    Compute,
}

impl WgpuContext {
    pub async fn new() -> Result<Self> {
        Self::open(ContextMode::Presentation).await
    }

    /// Compute-only device: no canvas, surface, or presentation pipeline.
    pub async fn new_compute() -> Result<Self> {
        Self::open(ContextMode::Compute).await
    }

    async fn open(mode: ContextMode) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let target = PresentationTarget::new();
        #[cfg(target_arch = "wasm32")]
        let target = PresentationTarget::new(mode)?;
        let adapter = target
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: target.surface.as_deref(),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find an appropriate adapter")?;
        report_adapter(&adapter);
        let (device, queue) = adapter
            .request_device(&Self::device_descriptor(&adapter))
            .await
            .context("Failed to create device")?;

        #[cfg(target_arch = "wasm32")]
        if let (Some(canvas), Some(surface)) = (&target.canvas, &target.surface) {
            let config = surface
                .get_default_config(&adapter, canvas.width(), canvas.height())
                .ok_or(BrowserContextError::SurfaceConfigurationUnavailable)?;
            surface.configure(&device, &config);
        }

        let mut context = Self {
            instance: Arc::new(target.instance),
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            #[cfg(target_arch = "wasm32")]
            canvas: target.canvas,
            surface: target.surface,
            blitter: None,
            blit_lock: Arc::new(async_lock::Mutex::new(())),
            blocking_validation: !cfg!(target_arch = "wasm32"),
        };
        if matches!(mode, ContextMode::Presentation) {
            context.blitter = Some(Arc::new(
                context
                    .observe_validation(|| {
                        Blitter::new(&context.device, wgpu::TextureFormat::Rgba8UnormSrgb)
                    })
                    .context("Failed to create presentation pipelines")?,
            ));
        }
        Ok(context)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PresentationTarget {
    fn new() -> Self {
        #[cfg(windows)]
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        #[cfg(not(windows))]
        let instance = Instance::default();
        Self {
            instance,
            surface: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl PresentationTarget {
    fn new(mode: ContextMode) -> Result<Self> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let (canvas, surface) = if matches!(mode, ContextMode::Presentation) {
            let canvas = web_sys::OffscreenCanvas::new(1, 1)
                .map_err(|cause| BrowserContextError::Canvas { cause })?;
            let surface = instance
                .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas.clone()))
                .context("Failed to create surface")?;
            (Some(canvas), Some(Arc::new(surface)))
        } else {
            (None, None)
        };
        Ok(Self {
            instance,
            surface,
            canvas,
        })
    }
}
