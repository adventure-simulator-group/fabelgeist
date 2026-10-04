//! One owner for a read mapping request and its eventual unmap or cancellation.

use crate::globals::WgpuContext;

use super::{MappedBufferBytes, ReadbackError};

/// A device view that borrows the owner responsible for unmapping it.
pub struct ReadbackView<'a> {
    view: wgpu::BufferView,
    owner: std::marker::PhantomData<&'a ()>,
}

impl ReadbackView<'_> {
    pub fn bytes(&self) -> MappedBufferBytes<'_> {
        MappedBufferBytes::from(&self.view[..])
    }
}

/// Releases a mapping request on failure or cancellation, and unmaps an acquired
/// mapping after its borrowed views are dropped.
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{ReadbackMapping, WgpuContext};
/// async fn unmap_while_borrowed(context: &WgpuContext, buffer: &wgpu::Buffer) {
///     let mapping = ReadbackMapping::new(context, buffer).await.unwrap();
///     let view = mapping.view().unwrap();
///     drop(mapping);
///     let _ = view.bytes();
/// }
/// ```
pub struct ReadbackMapping<'a> {
    buffer: &'a wgpu::Buffer,
}

impl<'a> ReadbackMapping<'a> {
    pub async fn new(
        context: &WgpuContext,
        buffer: &'a wgpu::Buffer,
    ) -> Result<Self, ReadbackError> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        buffer.slice(..).map_async(
            wgpu::MapMode::Read,
            move |result: Result<(), wgpu::BufferAsyncError>| {
                let _ = sender.send(result);
            },
        );
        let mapping = Self { buffer };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut receiver = receiver;
            context
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(ReadbackError::Poll)?;
            loop {
                match receiver.try_recv() {
                    Ok(Some(result)) => {
                        result.map_err(ReadbackError::Mapping)?;
                        break;
                    }
                    Ok(None) => {
                        context
                            .device
                            .poll(wgpu::PollType::Poll)
                            .map_err(ReadbackError::Poll)?;
                        std::thread::yield_now();
                    }
                    Err(source) => return Err(ReadbackError::MappingCanceled(source)),
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = context;
            receiver
                .await
                .map_err(ReadbackError::MappingCanceled)?
                .map_err(ReadbackError::Mapping)?;
        }
        Ok(mapping)
    }

    pub fn view(&self) -> Result<ReadbackView<'_>, ReadbackError> {
        Ok(ReadbackView {
            view: self
                .buffer
                .slice(..)
                .get_mapped_range()
                .map_err(ReadbackError::MappedRange)?,
            owner: std::marker::PhantomData,
        })
    }
}

impl Drop for ReadbackMapping<'_> {
    fn drop(&mut self) {
        self.buffer.unmap();
    }
}
