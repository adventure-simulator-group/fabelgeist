use super::*;

#[tokio::test]
async fn upload_preserves_native_words_and_checked_byte_offsets() {
    let context = WgpuContext::new().await.unwrap();
    let words = [0x80000000u32, 0x7fc01234, 0x7f800000, 0xbf800000];
    let upload = BufferUpload::from_elements(&words);
    let buffer = Buffer::from_upload(&context, upload, BufferDefinition::storage()).unwrap();
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), words);
    buffer
        .write_at(
            &context,
            4u64.into(),
            BufferUpload::from_elements(&[7u32, 8]),
        )
        .unwrap();
    assert_eq!(
        buffer.read::<u32>(&context).await.unwrap(),
        [words[0], 7, 8, words[3]]
    );
    assert_eq!(
        buffer.write_at(&context, 16u64.into(), BufferUpload::from_elements(&[1u32])),
        Err(BufferWriteError::OutOfBounds {
            at: 16u64.into(),
            bytes: 4u64.into(),
            length: 16u64.into(),
        })
    );
    assert_eq!(
        buffer.write_at(
            &context,
            u64::MAX.into(),
            BufferUpload::from_elements(&[1u32])
        ),
        Err(BufferWriteError::OffsetOverflow {
            at: u64::MAX.into(),
            bytes: 4u64.into(),
        })
    );
    assert_eq!(
        buffer.read::<u32>(&context).await.unwrap(),
        [words[0], 7, 8, words[3]]
    );
}

#[tokio::test]
async fn empty_upload_policy_is_explicit_at_the_consumer() {
    let context = WgpuContext::new().await.unwrap();
    let empty = BufferUpload::from_elements::<u32>(&[]);
    assert_eq!(empty.occupancy(), BufferUploadOccupancy::Empty);
    assert!(Buffer::from_upload(&context, empty, BufferDefinition::storage()).is_err());
    let binding = Buffer::from_upload(
        &context,
        empty.with_empty_word(),
        BufferDefinition::storage(),
    )
    .unwrap();
    assert_eq!(binding.read::<u32>(&context).await.unwrap(), [0]);
    let populated = BufferUpload::from_elements(&[42u32]);
    let binding = Buffer::from_upload(
        &context,
        populated.with_empty_word(),
        BufferDefinition::storage(),
    )
    .unwrap();
    assert_eq!(binding.read::<u32>(&context).await.unwrap(), [42]);
}
