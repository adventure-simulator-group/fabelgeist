//! Explicit queue completion, separate from command acceptance and readback.

use super::WgpuContext;
use crate::globals::CallbackProgressError;
#[cfg(target_arch = "wasm32")]
use crate::globals::await_browser_callback;
use std::{error::Error, fmt};

type Result<T> = std::result::Result<T, WorkCompletionError>;

/// The stage that prevented observing queue completion.
#[derive(Debug)]
pub enum WorkCompletionError {
    Marker(wgpu::Error),
    Poll(wgpu::PollError),
    Callback(futures_channel::oneshot::Canceled),
}

impl From<CallbackProgressError> for WorkCompletionError {
    fn from(cause: CallbackProgressError) -> Self {
        match cause {
            CallbackProgressError::Poll(cause) => Self::Poll(cause),
            CallbackProgressError::Canceled(cause) => Self::Callback(cause),
        }
    }
}

impl fmt::Display for WorkCompletionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Marker(cause) => write!(formatter, "GPU completion marker: {cause}"),
            Self::Poll(cause) => write!(formatter, "GPU completion progress: {cause}"),
            Self::Callback(cause) => write!(formatter, "GPU completion callback: {cause}"),
        }
    }
}

impl Error for WorkCompletionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Marker(cause) => Some(cause),
            Self::Poll(cause) => Some(cause),
            Self::Callback(cause) => Some(cause),
        }
    }
}

impl WgpuContext {
    /// Wait for work queued before this operation's completion point.
    ///
    /// The queue must be paired with this context's exact logical device.
    /// Native builds submit an empty marker and wait for its explicit index;
    /// later submissions need not finish. Marker Validation errors follow the
    /// context's synchronous observation policy; poll errors remain available.
    ///
    /// Browser builds await the SDK's queue callback. The WebGPU backend also
    /// delivers that callback when its underlying promise rejects, logging the
    /// rejection without exposing it here. `Ok` therefore establishes callback
    /// delivery only on that backend. Neither path certifies later work or
    /// global device health. Callers must first handle producing submissions.
    pub async fn submitted_work_done(&self) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let index = self
                .observe_validation(|| self.queue.submit([]))
                .map_err(WorkCompletionError::Marker)?;
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(index),
                    timeout: None,
                })
                .map_err(WorkCompletionError::Poll)?;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let (sender, mut receiver) = futures_channel::oneshot::channel();
            self.queue.on_submitted_work_done(move || {
                let _ = sender.send(());
            });
            await_browser_callback(&self.device, &mut receiver)
                .await
                .map_err(WorkCompletionError::from)
        }
    }
}
