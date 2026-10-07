use fabelgeist_gpu::prelude::*;
use std::error::Error as _;

#[tokio::test]
async fn checked_writes_retain_errors_bits_and_queue_order() {
    let context = WgpuContext::new_compute().await.unwrap();
    let words = [0x8000_0000u32, 0x7fc0_1234, 0x7f80_0000, 0xbf80_0000];
    let mut buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&words),
        BufferDefinition::storage(),
    )
    .unwrap();
    for offset in [BufferByteOffset::START, 4u64.into(), 16u64.into()] {
        buffer
            .write_at(&context, offset, BufferUpload::from_elements::<u32>(&[]))
            .unwrap();
    }
    buffer.size = 0u64.into();
    buffer
        .write_at(
            &context,
            BufferByteOffset::START,
            BufferUpload::from_elements::<u32>(&[]),
        )
        .unwrap();
    let overflow = buffer
        .write_at(
            &context,
            u64::MAX.into(),
            BufferUpload::from_elements(&[7u32]),
        )
        .unwrap_err();
    assert_eq!(
        overflow,
        BufferWriteError::OffsetOverflow {
            at: u64::MAX.into(),
            bytes: 4u64.into(),
        }
    );
    assert_eq!(overflow.to_string(), "Buffer write overflows an offset");
    assert!(overflow.source().is_none());
    buffer.size = 16u64.into();
    let float_words = words.map(f32::from_bits);
    buffer
        .write_at(
            &context,
            BufferByteOffset::START,
            BufferUpload::from_elements(&float_words),
        )
        .unwrap();
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), words);
    buffer
        .write_at(
            &context,
            BufferByteOffset::START,
            BufferUpload::from_elements(&[7u32]),
        )
        .unwrap();
    let bounds = buffer
        .write_at(
            &context,
            16u64.into(),
            BufferUpload::from_elements(&[999u32]),
        )
        .unwrap_err();
    assert_eq!(
        bounds,
        BufferWriteError::OutOfBounds {
            at: 16u64.into(),
            bytes: 4u64.into(),
            length: 16u64.into(),
        }
    );
    assert_eq!(
        bounds.to_string(),
        "Writing 4 bytes at 16 runs past the end of a 16-byte buffer"
    );
    assert!(bounds.source().is_none());
    let native = anyhow::Error::new(bounds);
    assert_eq!(native.downcast_ref::<BufferWriteError>(), Some(&bounds));
    assert!(native.downcast_ref::<String>().is_none());
    assert!(native.downcast_ref::<&'static str>().is_none());
    let outer = native.context("addressed fixture upload");
    assert_eq!(outer.downcast_ref::<BufferWriteError>(), Some(&bounds));
    assert_eq!(
        outer.downcast_ref::<&'static str>(),
        Some(&"addressed fixture upload")
    );
    assert_eq!(
        outer.root_cause().downcast_ref::<BufferWriteError>(),
        Some(&bounds)
    );
    assert_eq!(outer.chain().count(), 2);
    buffer
        .write_at(&context, 4u64.into(), BufferUpload::from_elements(&[8u32]))
        .unwrap();
    assert_eq!(
        buffer.read::<u32>(&context).await.unwrap(),
        [7, 8, words[2], words[3]]
    );
}

#[derive(Clone, Copy)]
enum NativePolicy {
    OffsetAlignment,
    LengthAlignment,
    DestinationUsage,
    AllocationCapacity,
}
#[derive(Clone, Copy)]
enum Admission {
    Accepted,
    Rejected,
}
struct SdkFixture {
    policy: NativePolicy,
    admission: Admission,
    logical_length: BufferByteLength,
    offset: BufferByteOffset,
    payload: &'static [u8],
}

#[tokio::test]
async fn sdk_checks_remain_after_logical_admission() {
    let context = WgpuContext::new_compute().await.unwrap();
    // Native literals construct byte leaves at this authored fixture boundary.
    let fixtures = [
        SdkFixture {
            policy: NativePolicy::OffsetAlignment,
            admission: Admission::Accepted,
            logical_length: 16u64.into(),
            offset: 2u64.into(),
            payload: &[1, 2, 3, 4],
        },
        SdkFixture {
            policy: NativePolicy::LengthAlignment,
            admission: Admission::Accepted,
            logical_length: 16u64.into(),
            offset: BufferByteOffset::START,
            payload: &[1, 2, 3],
        },
        SdkFixture {
            policy: NativePolicy::DestinationUsage,
            admission: Admission::Accepted,
            logical_length: 16u64.into(),
            offset: BufferByteOffset::START,
            payload: &[1, 2, 3, 4],
        },
        SdkFixture {
            policy: NativePolicy::AllocationCapacity,
            admission: Admission::Accepted,
            logical_length: 20u64.into(),
            offset: 16u64.into(),
            payload: &[1, 2, 3, 4],
        },
        SdkFixture {
            policy: NativePolicy::OffsetAlignment,
            admission: Admission::Rejected,
            logical_length: 0u64.into(),
            offset: 2u64.into(),
            payload: &[1, 2, 3, 4],
        },
        SdkFixture {
            policy: NativePolicy::LengthAlignment,
            admission: Admission::Rejected,
            logical_length: 0u64.into(),
            offset: BufferByteOffset::START,
            payload: &[1, 2, 3],
        },
        SdkFixture {
            policy: NativePolicy::DestinationUsage,
            admission: Admission::Rejected,
            logical_length: 0u64.into(),
            offset: BufferByteOffset::START,
            payload: &[1, 2, 3, 4],
        },
        SdkFixture {
            policy: NativePolicy::AllocationCapacity,
            admission: Admission::Rejected,
            logical_length: 16u64.into(),
            offset: 16u64.into(),
            payload: &[1, 2, 3, 4],
        },
    ];
    for fixture in fixtures {
        let definition = match fixture.policy {
            NativePolicy::DestinationUsage => BufferDefinition::copy_src(),
            _ => BufferDefinition::storage(),
        };
        let mut buffer = Buffer::new(&context, 16u64.into(), definition).unwrap();
        let native_length = buffer.size;
        buffer.size = fixture.logical_length;
        let scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let result = buffer.write_at(
            &context,
            fixture.offset,
            BufferUpload::from_elements(fixture.payload),
        );
        let write_error = scope.pop().await;
        let scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        context.queue.submit([]);
        let submit_error = scope.pop().await;
        match fixture.admission {
            Admission::Accepted => {
                result.unwrap();
                assert!(write_error.is_some() || submit_error.is_some());
            }
            Admission::Rejected => {
                assert!(matches!(result, Err(BufferWriteError::OutOfBounds { .. })));
                assert!(write_error.is_none() && submit_error.is_none());
            }
        }
        buffer.size = native_length;
        assert_eq!(buffer.read::<u32>(&context).await.unwrap(), [0; 4]);
    }
}
