//! Callback progress without turning an already delivered result into a poll.

use futures_channel::oneshot;
use std::{error::Error, fmt};

pub type CallbackResult<T> = Result<T, CallbackProgressError>;

/// Progress failed before a callback value could be received.
#[derive(Debug)]
pub enum CallbackProgressError {
    Poll(wgpu::PollError),
    Canceled(oneshot::Canceled),
}

impl fmt::Display for CallbackProgressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poll(cause) => write!(formatter, "GPU callback progress: {cause}"),
            Self::Canceled(cause) => write!(formatter, "GPU callback delivery: {cause}"),
        }
    }
}

impl Error for CallbackProgressError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Poll(cause) => Some(cause),
            Self::Canceled(cause) => Some(cause),
        }
    }
}

/// Take a delivered callback, or drive its device once without a GPU wait.
///
/// The receiver must belong to an operation on this device. Values are taken
/// once, following the receiver's contract. A ready value or closed channel
/// is observed before polling; neither becomes a test of later device health.
pub fn poll_callback<T>(
    device: &wgpu::Device,
    receiver: &mut oneshot::Receiver<T>,
) -> CallbackResult<Option<T>> {
    if let Some(value) = receiver
        .try_recv()
        .map_err(CallbackProgressError::Canceled)?
    {
        return Ok(Some(value));
    }
    device
        .poll(wgpu::PollType::Poll)
        .map_err(CallbackProgressError::Poll)?;
    receiver.try_recv().map_err(CallbackProgressError::Canceled)
}

/// Await a browser callback while yielding to its event loop between polls.
///
/// WebGPU progresses automatically; other enabled providers may require a
/// nonblocking device poll. A zero-duration timer yields a browser task turn
/// without imposing a polling interval. No native completion wait is issued.
#[cfg(target_arch = "wasm32")]
pub async fn await_browser_callback<T>(
    device: &wgpu::Device,
    receiver: &mut oneshot::Receiver<T>,
) -> CallbackResult<T> {
    loop {
        if let Some(value) = poll_callback(device, receiver)? {
            return Ok(value);
        }
        fabelgeist_timer::sleep(std::time::Duration::ZERO).await;
    }
}
