//! Public readback admission, native cause and reuse behavior.

use anyhow::Context;
use fabelgeist_gpu::prelude::{
    Buffer, BufferDefinition, BufferReadError, BufferReadResult, BufferUpload, BufferUse,
    GpuResource, WgpuContext,
};
use std::error::Error;

#[tokio::test]
async fn repeated_reads_preserve_native_bits_and_logical_padding() {
    let context = WgpuContext::new_compute().await.unwrap();
    let words = [0x80000000u32, 0x7fc01234, 0x7f800000, 0xff800000];
    for definition in [
        BufferDefinition::storage(),
        BufferDefinition::map_read().with_usage(BufferUse::CopyDestination),
    ] {
        let mut buffer =
            Buffer::from_upload(&context, BufferUpload::from_elements(&words), definition).unwrap();
        context.submitted_work_done().await;
        for _ in 0..2 {
            let read: BufferReadResult<Vec<u32>> = buffer.read(&context).await;
            assert_eq!(read.unwrap(), words);
            let floats = buffer.read::<f32>(&context).await.unwrap();
            assert_eq!(
                floats.into_iter().map(f32::to_bits).collect::<Vec<_>>(),
                words
            );
            assert_eq!(
                buffer.read::<[u32; 2]>(&context).await.unwrap(),
                [[words[0], words[1]], [words[2], words[3]]]
            );
        }
        buffer
            .write_at(&context, 4u64.into(), BufferUpload::from_elements(&[99u32]))
            .unwrap();
        context.submitted_work_done().await;
        assert_eq!(
            buffer.read::<u32>(&context).await.unwrap(),
            [words[0], 99, words[2], words[3]]
        );
        if buffer.usage.contains(wgpu::BufferUsages::MAP_READ) {
            buffer.size = 3u64.into();
            assert_eq!(
                buffer.read::<u8>(&context).await.unwrap(),
                words[0].to_ne_bytes()[..3]
            );
            buffer.size = 0u64.into();
            assert!(buffer.read::<u32>(&context).await.unwrap().is_empty());
        }
    }
}

#[tokio::test]
async fn host_rejections_and_truncated_views_preserve_reuse_and_context() {
    let context = WgpuContext::new_compute().await.unwrap();
    let mut buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[77u32]),
        BufferDefinition::map_read().with_usage(BufferUse::CopyDestination),
    )
    .unwrap();
    context.submitted_work_done().await;
    let scope = context
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    buffer.size = 3u64.into();
    assert!(matches!(
        buffer.read::<()>(&context).await,
        Err(BufferReadError::ZeroSizedElement)
    ));
    let error = buffer.read::<u32>(&context).await.unwrap_err();
    assert!(
        matches!(error, BufferReadError::PartialElement { bytes, element_bytes }
        if bytes == 3u64.into() && element_bytes == 4u64.into())
    );
    assert!(error.source().is_none());
    buffer.size = 8u64.into();
    let error = buffer.read::<u32>(&context).await.unwrap_err();
    assert!(
        matches!(error, BufferReadError::TruncatedMappedView { expected, available }
        if expected == 8u64.into() && available == 4u64.into())
    );
    assert_eq!(error.to_string(), "GPU readback is truncated");
    assert!(scope.pop().await.is_none());
    buffer.size = 4u64.into();
    buffer
        .write(&context, BufferUpload::from_elements(&[78u32]))
        .unwrap();
    context.submitted_work_done().await;
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), [78]);
    buffer.size = 3u64.into();
    let resource = GpuResource::from(buffer);
    let error = resource
        .read::<u32>(&context)
        .await
        .context("public resource read")
        .unwrap_err();
    assert_eq!(error.to_string(), "public resource read");
    assert!(matches!(
        error.downcast_ref::<BufferReadError>(),
        Some(BufferReadError::PartialElement { .. })
    ));
    assert!(
        error
            .root_cause()
            .downcast_ref::<BufferReadError>()
            .is_some()
    );
    assert!(error.root_cause().source().is_none());
    assert!(
        error
            .root_cause()
            .to_string()
            .contains("whole number of elements")
    );
}

#[tokio::test]
async fn mapping_rejection_retains_native_cause_and_source_words() {
    let context = WgpuContext::new_compute().await.unwrap();
    let mut buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[7u32, 11]),
        BufferDefinition::storage(),
    )
    .unwrap();
    let native_usage = buffer.usage;
    buffer.usage |= wgpu::BufferUsages::MAP_READ;
    let scope = context
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let error = buffer.read::<u32>(&context).await.unwrap_err();
    assert!(matches!(error, BufferReadError::Mapping(_)));
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(error.to_string(), "GPU Mapping error: BufferAsyncError");
    let cause = error.source().unwrap();
    assert!(cause.downcast_ref::<wgpu::BufferAsyncError>().is_some());
    assert!(cause.source().is_none());
    let rejection = scope.pop().await.unwrap();
    assert!(rejection.to_string().contains("Buffer::map_async"));
    buffer.usage = native_usage;
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), [7, 11]);
}

#[test]
fn dropped_native_sender_retains_cancellation_cause_at_admission_port() {
    let (sender, mut receiver) = futures_channel::oneshot::channel::<()>();
    drop(sender);
    let error = BufferReadError::CanceledChannel(receiver.try_recv().unwrap_err());
    assert_eq!(error.to_string(), "Mapping channel closed");
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<futures_channel::oneshot::Canceled>()
            .is_some()
    );
    let contextual = Err::<(), _>(error)
        .context("native cause admission")
        .unwrap_err();
    assert!(contextual.downcast_ref::<BufferReadError>().is_some());
    assert!(
        contextual
            .root_cause()
            .downcast_ref::<futures_channel::oneshot::Canceled>()
            .is_some()
    );
}
