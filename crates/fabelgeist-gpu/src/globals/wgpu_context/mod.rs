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
        wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
    } else {
        wgpu::Limits::default()
    };
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
    pub async fn new() -> Result<Self> {
        // Platform specific initialization
        #[cfg(target_arch = "wasm32")]
        let (canvas, surface, instance) = {
            let canvas = web_sys::OffscreenCanvas::new(1, 1)
                .map_err(|_| anyhow!("Failed to create OffscreenCanvas"))?;
            let desc = wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            };
            let instance = wgpu::util::new_instance_with_webgpu_detection(desc).await;
            let surface = instance
                .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas.clone()))
                .map_err(|_| anyhow!("Failed to create surface"))?;
            (Some(canvas), Some(Arc::new(surface)), instance)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (surface, instance): (Option<Arc<wgpu::Surface<'static>>>, Instance) = {
            // For native headless, we don't have a surface or canvas (unless we create a window, but we want headless)
            #[cfg(windows)]
            let instance = Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::DX12,
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
                apply_limit_buckets: false,
            })
            .await
            .map_err(|_| anyhow!("Failed to find an appropriate adapter"))?;

        report_adapter(&adapter);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: optional_features(&adapter),
                required_limits: required_limits(&adapter),
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: Default::default(),
            })
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
        let blitter = Some(Arc::new(Blitter::new(
            &device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )));

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
        })
    }
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for WgpuContext {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WgpuContext {}
