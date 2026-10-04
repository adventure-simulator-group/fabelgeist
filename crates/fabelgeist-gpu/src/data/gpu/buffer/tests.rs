use super::{write_error::BufferWriteRange, *};

#[derive(serde::Deserialize)]
struct OriginalWriteCase {
    offset: u64,
    bytes: u64,
    available: u64,
    error: Option<String>,
}

#[test]
fn logical_write_admission_preserves_original_results_and_diagnostics() {
    let cases: Vec<OriginalWriteCase> = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/buffer_writes.json"
    ))
    .unwrap();
    for case in cases {
        let at = BufferByteOffset::from(case.offset);
        let bytes = BufferByteLength::from(case.bytes);
        let length = BufferByteLength::from(case.available);
        match (BufferWriteRange::new(at, bytes, length), case.error) {
            (Ok(_), None) => {}
            (Err(error), Some(expected)) => {
                assert_eq!(error.to_string(), expected);
                match error {
                    BufferWriteError::OffsetOverflow {
                        at: actual,
                        bytes: actual_bytes,
                    } => {
                        assert_eq!((actual, actual_bytes), (at, bytes));
                    }
                    BufferWriteError::OutOfBounds {
                        at: actual,
                        bytes: actual_bytes,
                        length: actual_length,
                    } => assert_eq!((actual, actual_bytes, actual_length), (at, bytes, length)),
                }
            }
            _ => panic!("write admission differs from the original fixture"),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn logical_lengths_and_offset_writes_keep_allocation_and_contents() {
    let context = WgpuContext::new().await.unwrap();
    let buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[7u32, 11]),
        BufferDefinition::storage(),
    )
    .unwrap();
    let length = BufferByteLength::from(8u64);
    assert_eq!(buffer.length(), length);
    assert!(matches!(
        buffer.clone().with_logical_length(9u64.into()),
        Err(BufferLengthError { requested, available })
            if requested == 9u64.into() && available == length
    ));
    let shortened = buffer.clone().with_logical_length(4u64.into()).unwrap();
    assert_eq!(shortened.read::<u32>(&context).await.unwrap(), [7]);
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), [7, 11]);
    let empty = buffer
        .clone()
        .with_logical_length(BufferByteLength::default())
        .unwrap();
    assert!(empty.read::<u32>(&context).await.unwrap().is_empty());
    assert_eq!(empty.with_logical_length(length).unwrap(), buffer);

    assert!(matches!(
        shortened.write_at(&context, 4u64.into(),BufferUpload::from_elements(&[13u32])),
        Err(BufferWriteError::OutOfBounds { at, bytes, length })
            if at == 4u64.into() && bytes == 4u64.into() && length == 4u64.into()
    ));
    assert!(matches!(
        buffer.write_at(
            &context,
            u64::MAX.into(),
            BufferUpload::from_elements(&[13u32])
        ),
        Err(BufferWriteError::OffsetOverflow { .. })
    ));
    buffer
        .write_at(&context, 4u64.into(), BufferUpload::from_elements(&[13u32]))
        .unwrap();
    buffer
        .write_at(
            &context,
            8u64.into(),
            BufferUpload::from_elements(&[] as &[u32]),
        )
        .unwrap();
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), [7, 13]);
    assert!(matches!(
        Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[] as &[u8]),
            BufferDefinition::storage()
        ),
        Err(BufferCreationError::Empty)
    ));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn borrowed_uploads_preserve_float_bits_and_empty_binding_words() {
    let context = WgpuContext::new().await.unwrap();
    let bits = [0x8000_0000u32, 0x7fc0_1234, 0x3f80_0000, 0xff80_0000];
    let values = bits.map(f32::from_bits);
    let upload = BufferUpload::from_elements(&values);
    let buffer = Buffer::from_upload(&context, upload, BufferDefinition::storage()).unwrap();
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), bits);
    assert!(buffer.buffer.usage().contains(wgpu::BufferUsages::COPY_DST));
    let replacement = [0u32, 1, 2, 3];
    buffer.write(&context, BufferUpload::from_elements(&replacement));
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), replacement);

    let empty = BufferUpload::from_elements(&[] as &[f32]);
    let placeholder = Buffer::from_upload(
        &context,
        empty.with_empty_word(),
        BufferDefinition::storage(),
    )
    .unwrap();
    assert_eq!(placeholder.read::<u32>(&context).await.unwrap(), [0]);
    assert!(matches!(
        Buffer::from_upload(&context, empty, BufferDefinition::storage()),
        Err(BufferCreationError::Empty)
    ));
}
