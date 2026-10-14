use fabelgeist_gpu::prelude::{
    Buffer, BufferCreationError, BufferCreationResult, BufferDefinition, BufferUpload, BufferUse,
    WgpuContext,
};
use std::error::Error;
use std::sync::Arc;

#[tokio::test]
async fn zero_rejection_precedes_native_allocation_and_preserves_context() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let native_scope = context
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let error = Buffer::new(
        &context,
        0u64.into(),
        BufferDefinition::storage().with_usage(BufferUse::HostRead),
    )
    .expect_err("zero precedes invalid native capability");
    assert_eq!(error, BufferCreationError::Empty);
    assert_eq!(error.to_string(), "Buffer size must be greater than 0");
    assert_eq!(format!("{error:?}"), "Empty");
    assert!(error.source().is_none());
    assert!(native_scope.pop().await.is_none());

    let contextual = anyhow::Error::from(error).context("buffer creation context");
    assert_eq!(contextual.to_string(), "buffer creation context");
    assert_eq!(
        contextual.downcast_ref::<BufferCreationError>(),
        Some(&error)
    );
    assert_eq!(
        contextual.downcast_ref::<&'static str>(),
        Some(&"buffer creation context")
    );
    assert_eq!(contextual.root_cause().to_string(), error.to_string());
    assert!(contextual.root_cause().source().is_none());
    assert!(contextual.downcast_ref::<String>().is_none());
    Ok(())
}

#[tokio::test]
async fn empty_uploads_reject_but_explicit_word_padding_allocates() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    for upload in [
        BufferUpload::from_elements::<u32>(&[]),
        BufferUpload::from_elements(&[(); 3]),
    ] {
        let rejected: BufferCreationResult<Buffer> =
            Buffer::from_upload(&context, upload, BufferDefinition::storage());
        assert_eq!(
            rejected.expect_err("zero-byte upload"),
            BufferCreationError::Empty
        );
        let padded = Buffer::from_upload(
            &context,
            upload.with_empty_word(),
            BufferDefinition::storage(),
        )?;
        assert_eq!(u64::from(padded.size), 4);
        assert_eq!(padded.read::<u32>(&context).await?, [0]);
    }
    Ok(())
}

#[tokio::test]
async fn successful_upload_preserves_bits_capability_and_native_sharing() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    // Native f32 representation includes signed zero, a NaN payload and infinity.
    let words = [0x80000000u32, 0x7fc01234, 0x7f800000, 0xff800000];
    let buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&words),
        BufferDefinition::index().with_usage(BufferUse::CopySource),
    )?;
    assert_eq!(u64::from(buffer.size), 16);
    assert_eq!(buffer.buffer.size(), 16);
    assert_eq!(buffer.read::<u32>(&context).await?, words);
    assert_eq!(
        buffer.buffer.usage(),
        wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST
    );
    let clone = buffer.clone();
    assert_eq!(buffer, clone);
    assert!(Arc::ptr_eq(&buffer.buffer, &clone.buffer));
    let mut logical_empty = clone;
    logical_empty.size = 0u64.into();
    assert!(logical_empty.size.is_empty());
    assert_ne!(logical_empty, buffer);
    assert!(Arc::ptr_eq(&logical_empty.buffer, &buffer.buffer));
    Ok(())
}

#[tokio::test]
async fn nonzero_native_upload_rejection_stays_provider_owned() -> anyhow::Result<()> {
    let context = WgpuContext::new().await?;
    let native_scope = context
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let result = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[1u8]),
        BufferDefinition::storage(),
    );
    assert!(
        result.is_ok(),
        "native alignment is outside creation admission"
    );
    assert!(native_scope.pop().await.is_some());
    Ok(())
}
