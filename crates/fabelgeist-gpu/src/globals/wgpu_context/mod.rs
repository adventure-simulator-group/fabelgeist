use anyhow::{Result, anyhow};
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue};
pub mod blitter;
pub use blitter::Blitter;

#[derive(Clone)]
pub struct WgpuContext {
    pub instance: Arc<Instance>,
    pub adapter: Arc<Adapter>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    #[cfg(target_arch = "wasm32")]
    pub canvas: Option<web_sys::OffscreenCanvas>,
    pub surface: Option<Arc<wgpu::Surface<'static>>>,
    pub blitter: Option<Arc<Blitter>>,
    pub blit_lock: Arc<async_lock::Mutex<()>>,
    /// Whether shader, pipeline and texture construction synchronously waits
    /// for native validation results.
    ///
    /// Native contexts created here enable this; borrowed contexts disable it
    /// to avoid blocking their caller on GPU work. A blocking poll waits for
    /// the latest submission observed when polling begins and has no timeout.
    /// When disabled, these constructors install no validation scope, so
    /// native errors follow the device's configured error handling. Browser
    /// builds skip synchronous readback. Shader source is separately checked
    /// by Naga.
    pub blocking_validation: bool,
}

/// Reports which GPU the adapter request actually landed on. `device_type: Cpu`
/// means a software fallback (SwiftShader); `backend: Vulkan` with
/// `device_type: DiscreteGpu` is the intended discrete card.
fn report_adapter(adapter: &Adapter) {
    let info = adapter.get_info();
    let message = format!(
        "[fabelgeist-gpu] adapter: {} | type: {:?} | backend: {:?} | driver: {} {}",
        info.name, info.device_type, info.backend, info.driver, info.driver_info
    );
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&message.as_str().into());
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{message}");
}

/// The optional features we use where the adapter offers them. Each is a
/// graceful degradation, not a requirement, so requesting one the adapter
/// lacks would fail device creation outright.
fn optional_features(adapter: &Adapter) -> wgpu::Features {
    const OPTIONAL: [wgpu::Features; 3] = [
        wgpu::Features::FLOAT32_FILTERABLE,
        wgpu::Features::FLOAT32_BLENDABLE,
        wgpu::Features::TIMESTAMP_QUERY,
    ];
    OPTIONAL
        .into_iter()
        .filter(|feature| adapter.features().contains(*feature))
        .fold(wgpu::Features::empty(), |features, feature| {
            features | feature
        })
}

/// Raises the baseline limits to what the adapter actually supports. The
/// downlevel/WebGL2 baseline caps compute invocations at 256, but our kernels
/// dispatch workgroups of up to 1024 invocations.
fn required_limits(adapter: &Adapter) -> wgpu::Limits {
    let adapter_limits = adapter.limits();
    let mut limits = if cfg!(target_arch = "wasm32") {
        wgpu::Limits::default().using_resolution(adapter.limits())
    } else {
        wgpu::Limits::default()
    };
    limits.max_storage_buffers_per_shader_stage =
        adapter_limits.max_storage_buffers_per_shader_stage;
    limits.max_buffer_size = adapter_limits.max_buffer_size;
    limits.max_storage_buffer_binding_size = adapter_limits.max_storage_buffer_binding_size;
    limits.max_compute_invocations_per_workgroup =
        adapter_limits.max_compute_invocations_per_workgroup;
    limits.max_compute_workgroup_size_x = adapter_limits.max_compute_workgroup_size_x;
    limits.max_compute_workgroup_size_y = adapter_limits.max_compute_workgroup_size_y;
    limits.max_compute_workgroup_size_z = adapter_limits.max_compute_workgroup_size_z;
    limits.max_compute_workgroups_per_dimension =
        adapter_limits.max_compute_workgroups_per_dimension;
    limits
}

impl WgpuContext {
    /// Wrap a device that already exists.
    ///
    /// An application that has its own wgpu setup -- a renderer, a compositor
    /// -- wants the solver on that same device, so that simulated positions
    /// can be drawn straight from the buffer they were written into rather
    /// than copied between two devices. There is no surface and no blitter:
    /// those belong to whoever owns the window.
    pub fn from_parts(
        instance: Arc<Instance>,
        adapter: Arc<Adapter>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self {
        Self {
            instance,
            adapter,
            device,
            queue,
            #[cfg(target_arch = "wasm32")]
            canvas: None,
            surface: None,
            blitter: None,
            blit_lock: Arc::new(async_lock::Mutex::new(())),
            // Someone else's device, and very likely someone else's render
            // loop with it.
            blocking_validation: false,
        }
    }

    /// Wait until a queue marker and all work submitted before it complete.
    ///
    /// Callers can use this for backpressure before submitting more work.
    /// Native builds wait on an explicit submission index; submissions made
    /// after that marker need not finish before this call returns. Browser
    /// builds wait for the queue's completion callback instead.
    pub async fn submitted_work_done(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Wait for this point in the queue only. A submission index
            // returns once earlier work finishes even while a renderer keeps
            // submitting, and without a sleep's timer granularity, which on
            // Windows is the system tick rather than a millisecond.
            let index = self.queue.submit([]);
            let _ = self.device.poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            });
        }
        // WebGPU ignores `Wait`; its callbacks run from the event loop.
        #[cfg(target_arch = "wasm32")]
        {
            let (sender, mut receiver) = futures_channel::oneshot::channel();
            self.queue.on_submitted_work_done(move || {
                let _ = sender.send(());
            });
            loop {
                let _ = self.device.poll(wgpu::PollType::Poll);
                match receiver.try_recv() {
                    Ok(Some(())) => return,
                    Ok(None) => {
                        fabelgeist_timer::sleep(std::time::Duration::from_millis(1)).await;
                    }
                    // The callback was dropped, which can only happen if the
                    // device is going away. Nothing left to wait for.
                    Err(_) => return,
                }
            }
        }
    }

    /// What the GPU crates ask of a device: filterable and blendable float textures
    /// and timestamp queries where the adapter has them, and the adapter's
    /// own buffer and compute limits, which the kernels need beyond wgpu's
    /// defaults (WebGL2's, in a browser).
    pub fn device_descriptor(adapter: &Adapter) -> wgpu::DeviceDescriptor<'static> {
        wgpu::DeviceDescriptor {
            label: None,
            required_features: optional_features(adapter),
            required_limits: required_limits(adapter),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::default(),
            trace: Default::default(),
        }
    }

    pub async fn new() -> Result<Self> {
        Self::open(true).await
    }

    /// Compute-only device: no canvas, surface, or presentation pipeline.
    pub async fn new_compute() -> Result<Self> {
        Self::open(false).await
    }

    async fn open(presentation: bool) -> Result<Self> {
        // Platform specific initialization
        #[cfg(target_arch = "wasm32")]
        let (canvas, surface, instance) = {
            let desc = wgpu::InstanceDescriptor {
                backends: wgpu::Backends::BROWSER_WEBGPU,
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            };
            let instance = Instance::new(desc);
            if presentation {
                let canvas = web_sys::OffscreenCanvas::new(1, 1)
                    .map_err(|_| anyhow!("Failed to create OffscreenCanvas"))?;
                let surface = instance
                    .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas.clone()))
                    .map_err(|_| anyhow!("Failed to create surface"))?;
                (Some(canvas), Some(Arc::new(surface)), instance)
            } else {
                (None, None, instance)
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (surface, instance): (Option<Arc<wgpu::Surface<'static>>>, Instance) = {
            // For native headless, we don't have a surface or canvas (unless we create a window, but we want headless)
            #[cfg(windows)]
            let instance = Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            });

            #[cfg(not(windows))]
            let instance = Instance::default();

            (None, instance)
        };

        let compatible_surface = surface
            .as_ref()
            .map(|s: &Arc<wgpu::Surface<'static>>| s.as_ref());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| anyhow!("Failed to find an appropriate adapter"))?;

        report_adapter(&adapter);

        let (device, queue) = adapter
            .request_device(&Self::device_descriptor(&adapter))
            .await
            .map_err(|_| anyhow!("Failed to create device"))?;

        #[cfg(target_arch = "wasm32")]
        if let (Some(canvas), Some(surface)) = (&canvas, &surface) {
            let width = canvas.width();
            let height = canvas.height();
            let config = surface.get_default_config(&adapter, width, height).unwrap();
            surface.configure(&device, &config);
        }

        // Blit pipeline (used by Texture3d widget)
        // We use Rgba8UnormSrgb because DisplayTexture3dWidget creates its temporary 2D texture in this format
        let blitter = presentation
            .then(|| Arc::new(Blitter::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb)));

        Ok(WgpuContext {
            instance: Arc::new(instance),
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            #[cfg(target_arch = "wasm32")]
            canvas,
            surface,
            blitter,
            blit_lock: Arc::new(async_lock::Mutex::new(())),
            blocking_validation: !cfg!(target_arch = "wasm32"),
        })
    }
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for WgpuContext {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WgpuContext {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Borrowed contexts leave synchronous validation to the application.
    ///
    /// Construction paths consult this flag before waiting for native
    /// validation results. Wrapping an application's device must preserve the
    /// nonblocking policy even when the original context permits a wait.
    #[tokio::test]
    async fn a_borrowed_device_skips_blocking_validation() -> Result<()> {
        let owned = WgpuContext::new().await?;
        assert!(
            owned.blocking_validation,
            "a context that owns its device can afford to read the error scope"
        );

        let borrowed = WgpuContext::from_parts(
            owned.instance.clone(),
            owned.adapter.clone(),
            owned.device.clone(),
            owned.queue.clone(),
        );
        assert!(
            !borrowed.blocking_validation,
            "wrapping a device must not enable synchronous validation readback"
        );
        assert!(borrowed.surface.is_none());
        Ok(())
    }
}
