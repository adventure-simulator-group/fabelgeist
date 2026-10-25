#[cfg(test)]
use anyhow::Result;
pub use blitter::Blitter;
pub use completion::WorkCompletionError;
#[cfg(target_arch = "wasm32")]
pub use initialization_error::BrowserContextError;
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue};

pub mod blitter;
mod completion;
mod initialization;
#[cfg(target_arch = "wasm32")]
mod initialization_error;
mod submission;
mod validation;

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
    /// Whether native construction, encoder finalization and submission
    /// observe Validation errors synchronously.
    ///
    /// Native contexts created here enable this by default. `from_parts`
    /// disables it by default to avoid blocking its caller on GPU work.
    /// A blocking poll waits for the latest submission observed when polling
    /// begins, requesting no explicit timeout. A backend can still fail that
    /// wait; observing construction propagates the actual polling error and
    /// any captured validation error.
    /// Finalization and submission read their scopes without a completion wait.
    /// When disabled, these operations install no verdict scope, so
    /// native errors follow the device's configured error handling. Browser
    /// builds skip synchronous readback. Shader source is separately checked
    /// by Naga. Failed or cancelled buffer mapping uses a native cleanup-only
    /// scope, including on borrowed contexts, to reset SDK mapping state
    /// without replacing the primary error with a cleanup error. It does not
    /// wait for GPU work or establish an operation verdict.
    pub blocking_validation: bool,
}

impl WgpuContext {
    /// Wrap a device that already exists.
    ///
    /// `queue` must be paired with `device` for operations using this context
    /// to submit work or upload data.
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
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for WgpuContext {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WgpuContext {}

/// Reports the selected adapter, device type, backend and driver. A CPU
/// adapter reports software rendering; its driver identifies the implementation.
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
