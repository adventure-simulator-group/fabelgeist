//! Public batch-copy admission and ordering, with real GPU readback.

use anyhow::Result;
use fabelgeist_compute::{BufferCopyError, Kernel, KernelBatch};
use fabelgeist_gpu::prelude::*;

#[derive(Clone, Copy, Debug)]
enum CopyAdmission {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug)]
struct CopyFixture {
    source_length: BufferByteLength,
    destination_length: BufferByteLength,
    bytes: BufferByteLength,
    admission: CopyAdmission,
}

impl CopyFixture {
    fn new(
        source_length: BufferByteLength,
        destination_length: BufferByteLength,
        bytes: BufferByteLength,
        admission: CopyAdmission,
    ) -> Self {
        Self {
            source_length,
            destination_length,
            bytes,
            admission,
        }
    }
}

#[test]
fn logical_extents_admit_copies_and_reject_without_changing_tails() -> Result<()> {
    pollster::block_on(async {
        let context = WgpuContext::new_compute().await?;
        use CopyAdmission::{Accepted, Rejected};
        // Native byte literals enter typed fields at this authored fixture port.
        let fixtures = [
            CopyFixture::new(16u64.into(), 16u64.into(), 0u64.into(), Accepted),
            CopyFixture::new(16u64.into(), 16u64.into(), 4u64.into(), Accepted),
            CopyFixture::new(16u64.into(), 16u64.into(), 16u64.into(), Accepted),
            CopyFixture::new(16u64.into(), 8u64.into(), 8u64.into(), Accepted),
            CopyFixture::new(8u64.into(), 16u64.into(), 12u64.into(), Rejected),
            CopyFixture::new(16u64.into(), 8u64.into(), 12u64.into(), Rejected),
            CopyFixture::new(4u64.into(), 8u64.into(), 12u64.into(), Rejected),
            CopyFixture::new(0u64.into(), 0u64.into(), 0u64.into(), Accepted),
            CopyFixture::new(32u64.into(), 32u64.into(), 16u64.into(), Accepted),
            CopyFixture::new(16u64.into(), 16u64.into(), u64::MAX.into(), Rejected),
            CopyFixture::new(u64::MAX.into(), 16u64.into(), 20u64.into(), Rejected),
        ];
        for fixture in fixtures {
            let source_words = [0x8000_0000u32, 0x7fc0_1234, 1, u32::MAX];
            let destination_words = [0xaaaa_aaaau32, 0xbbbb_bbbb, 0xcccc_cccc, 0xdddd_dddd];
            let mut source = Buffer::from_upload(
                &context,
                BufferUpload::from_elements(&source_words),
                BufferDefinition::storage(),
            )?;
            let mut destination = Buffer::from_upload(
                &context,
                BufferUpload::from_elements(&destination_words),
                BufferDefinition::storage(),
            )?;
            let readback = destination.clone();
            source.size = fixture.source_length;
            destination.size = fixture.destination_length;
            let mut batch = KernelBatch::new(&context);
            let result = batch.copy_buffer(&source, &destination, fixture.bytes);
            let admitted = result.is_ok();
            assert_eq!(
                admitted,
                matches!(fixture.admission, Accepted),
                "{fixture:?}"
            );
            if let Err(error) = result {
                assert_eq!(error.bytes, fixture.bytes);
                assert_eq!(error.source_length, fixture.source_length);
                assert_eq!(error.destination_length, fixture.destination_length);
                assert!(std::error::Error::source(&error).is_none());
                let error = anyhow::Error::from(error);
                assert!(error.downcast_ref::<BufferCopyError>().is_some());
                assert!(error.downcast_ref::<String>().is_none());
            }
            assert_eq!(batch.dispatch_count(), 0);
            batch.submit();
            let read: Vec<u32> = readback.read(&context).await?;
            let mut expected = destination_words;
            if admitted {
                // Native readback is an array of four-byte words.
                let words = usize::try_from(u64::from(fixture.bytes) / 4)?;
                expected[..words].copy_from_slice(&source_words[..words]);
            }
            assert_eq!(read, expected, "{fixture:?}");
        }
        Ok(())
    })
}

const INCREMENT: &str = r#"
@group(0) @binding(0) var<storage, read_write> values: array<u32>;
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < arrayLength(&values)) {
        values[id.x] += 1u;
    }
}
"#;

#[test]
fn rejection_preserves_work_between_dependent_dispatches() -> Result<()> {
    pollster::block_on(async {
        let context = WgpuContext::new_compute().await?;
        let source = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[3u32, 7, 11, 19]),
            BufferDefinition::storage(),
        )?;
        let destination = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[0u32; 4]),
            BufferDefinition::storage(),
        )?;
        let kernel = Kernel::new(&context, INCREMENT)?;
        let mut parameters = PassParameters::new();
        parameters.insert("values", source.clone());
        let mut batch = KernelBatch::new(&context);
        batch.dispatch(&kernel, &parameters, [4, 1, 1])?;
        let error = batch
            .copy_buffer(&source, &destination, 20u64.into())
            .expect_err("a twenty-byte copy cannot fit sixteen-byte buffers");
        assert_eq!(
            error.to_string(),
            "KernelBatch::copy_buffer: 20 bytes does not fit 16 -> 16"
        );
        assert_eq!(batch.dispatch_count(), 1);
        batch.copy_buffer(&source, &destination, 16u64.into())?;
        assert_eq!(batch.dispatch_count(), 1);
        parameters.insert("values", destination.clone());
        batch.dispatch(&kernel, &parameters, [4, 1, 1])?;
        assert_eq!(batch.dispatch_count(), 2);
        batch.submit();
        let source_read: Vec<u32> = source.read(&context).await?;
        let destination_read: Vec<u32> = destination.read(&context).await?;
        assert_eq!(source_read, [4, 8, 12, 20]);
        assert_eq!(destination_read, [5, 9, 13, 21]);
        Ok(())
    })
}

#[derive(Clone, Copy, Debug)]
enum SdkControl {
    Alignment,
    SourceUsage,
    DestinationUsage,
    SameBuffer,
}

#[test]
fn native_validation_remains_after_logical_extent_admission() -> Result<()> {
    pollster::block_on(async {
        let context = WgpuContext::new_compute().await?;
        for control in [
            SdkControl::Alignment,
            SdkControl::SourceUsage,
            SdkControl::DestinationUsage,
            SdkControl::SameBuffer,
        ] {
            let source_definition = if matches!(control, SdkControl::SourceUsage) {
                BufferDefinition::uniform()
            } else {
                BufferDefinition::storage()
            };
            let destination_definition = if matches!(control, SdkControl::DestinationUsage) {
                BufferDefinition::uniform()
            } else {
                BufferDefinition::storage()
            };
            let source = Buffer::new(&context, 16u64.into(), source_definition)?;
            let destination = if matches!(control, SdkControl::SameBuffer) {
                source.clone()
            } else {
                Buffer::new(&context, 16u64.into(), destination_definition)?
            };
            let bytes = if matches!(control, SdkControl::Alignment) {
                1u64.into()
            } else {
                4u64.into()
            };
            let scope = context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation);
            let mut batch = KernelBatch::new(&context);
            batch.copy_buffer(&source, &destination, bytes)?;
            assert_eq!(batch.dispatch_count(), 0);
            let recording_error = scope.pop().await;
            let scope = context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation);
            batch.submit();
            let finish_error = scope.pop().await;
            assert!(
                recording_error.is_some() || finish_error.is_some(),
                "SDK validation required for {control:?}"
            );
        }
        Ok(())
    })
}
