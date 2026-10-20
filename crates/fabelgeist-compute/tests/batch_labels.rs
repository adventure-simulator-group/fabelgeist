#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch, KernelBatchLabel};
use fabelgeist_gpu::prelude::*;

enum DiagnosticFixture {
    Default,
    Named(&'static str),
}

#[tokio::test]
async fn native_diagnostics_preserve_admitted_spelling() -> Result<()> {
    let context = WgpuContext::new().await?;
    let source = Buffer::new(
        &context,
        4u64.into(),
        BufferDefinition::uniform().with_label("label preflight source".into()),
    )?;
    let destination = Buffer::new(&context, 4u64.into(), BufferDefinition::storage())?;
    let cases = [
        DiagnosticFixture::Default,
        DiagnosticFixture::Named(""),
        DiagnosticFixture::Named(" Δ batch::namespace "),
        DiagnosticFixture::Named("batch/plain"),
        DiagnosticFixture::Named(" \t label \n "),
        DiagnosticFixture::Named("label\u{1}end"),
        DiagnosticFixture::Named("before\0after"),
    ];
    for case in cases {
        let spelling = match case {
            DiagnosticFixture::Default => "KernelBatch",
            DiagnosticFixture::Named(spelling) => spelling,
        };
        let mut batch = match case {
            DiagnosticFixture::Default => KernelBatch::new(&context),
            DiagnosticFixture::Named(spelling) => {
                KernelBatch::labelled(&context, KernelBatchLabel::from(spelling))
            }
        };
        let recording = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        batch
            .encoder()
            .copy_buffer_to_buffer(&source.buffer, 0, &destination.buffer, 0, 4);
        assert!(recording.pop().await.is_none());
        let submission = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        batch.submit();
        let error = submission
            .pop()
            .await
            .expect("native finish rejects missing COPY_SRC");
        assert!(matches!(error, wgpu::Error::Validation { .. }));
        let encoder = if spelling.is_empty() {
            "In a CommandEncoder".to_owned()
        } else {
            format!("In a CommandEncoder, label = '{spelling}'")
        };
        assert_eq!(
            error.to_string(),
            format!(
                "Validation Error\n\nCaused by:\n  {encoder}\n    Usage flags BufferUsages(UNIFORM) of Buffer with 'label preflight source' label do not contain required usage flags BufferUsages(COPY_SRC)\n"
            )
        );
        let words: Vec<u32> = destination.read(&context).await?;
        assert_eq!(words, [0]);
    }
    Ok(())
}

#[tokio::test]
async fn local_label_can_drop_before_ordered_batch_work() -> Result<()> {
    let context = WgpuContext::new().await?;
    let initial = [0u32, 1, u32::MAX];
    let source = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&initial),
        BufferDefinition::storage(),
    )?;
    let destination = Buffer::new(&context, 12u64.into(), BufferDefinition::storage())?;
    let kernel = Kernel::new(
        &context,
        r#"
@group(0) @binding(0) var<storage, read_write> values: array<u32>;
@compute @workgroup_size(1) fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    values[id.x] = values[id.x] + 1u;
}
"#,
    )?;
    let mut parameters = PassParameters::new();
    parameters.insert("values", destination.clone());
    let mut batch = {
        let local_label = String::from(" Δ local label\0 ");
        KernelBatch::labelled(&context, KernelBatchLabel::from(local_label.as_str()))
    };
    batch.clear_buffer(&destination);
    batch.copy_buffer(&source, &destination, 12)?;
    batch.dispatch(&kernel, &parameters, [0, 1, 1])?;
    assert_eq!(batch.dispatch_count(), 0);
    batch.dispatch(&kernel, &parameters, [3, 1, 1])?;
    batch.dispatch(&kernel, &parameters, [3, 1, 1])?;
    assert_eq!(batch.dispatch_count(), 2);
    batch.clear_buffer(&source);
    batch.submit();
    let output: Vec<u32> = destination.read(&context).await?;
    let cleared: Vec<u32> = source.read(&context).await?;
    assert_eq!(output, [2, 3, 1]);
    assert_eq!(cleared, [0, 0, 0]);
    Ok(())
}
