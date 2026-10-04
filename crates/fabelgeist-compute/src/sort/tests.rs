use super::*;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
struct FixtureKey(u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
struct FixturePayload(u32);
#[derive(Clone, Debug)]
struct SortFixture(Vec<FixtureKey>);
impl From<&[u32]> for SortFixture {
    fn from(words: &[u32]) -> Self {
        Self(words.iter().copied().map(FixtureKey).collect())
    }
}
struct FixtureState(u32);
impl From<u32> for FixtureState {
    fn from(word: u32) -> Self {
        Self(word | 1)
    }
}
impl SortFixture {
    fn generated(count: SortItemCount, mut state: FixtureState) -> Self {
        let mut keys = Vec::new();
        for _ in 0..u64::from(count.word_bytes()) / 4 {
            state.0 ^= state.0 << 13;
            state.0 ^= state.0 >> 17;
            state.0 ^= state.0 << 5;
            keys.push(FixtureKey(state.0));
        }
        Self(keys)
    }
}
struct SortOutput {
    keys: Vec<FixtureKey>,
    values: Vec<FixturePayload>,
}

async fn sorted(keys: &SortFixture, bits: SortKeyWidth) -> Result<SortOutput> {
    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;

    let count = SortItemCount::from(keys.0.len() as u32);
    // The payload is each key's original index, so the assertions can check
    // that values travelled with their keys.
    let values: Vec<FixturePayload> = (0..keys.0.len() as u32).map(FixturePayload).collect();

    let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
    let key_buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&keys.0),
        definition.clone(),
    )?;
    let value_buffer =
        Buffer::from_upload(&context, BufferUpload::from_elements(&values), definition)?;
    let mut scratch = SortScratch::new(&context, count)?;

    sort.run(
        &context,
        &key_buffer,
        &value_buffer,
        &mut scratch,
        count,
        bits,
    )?;

    Ok(SortOutput {
        keys: key_buffer.read(&context).await?,
        values: value_buffer.read(&context).await?,
    })
}

/// What the GPU should have produced: a stable sort by key.
fn reference(input: &SortFixture) -> SortOutput {
    let mut pairs: Vec<(FixtureKey, FixturePayload)> = input
        .0
        .iter()
        .copied()
        .zip((0u32..).map(FixturePayload))
        .collect();
    pairs.sort_by_key(|&(key, _): &(FixtureKey, FixturePayload)| -> FixtureKey { key });
    let (keys, values) = pairs.into_iter().unzip();
    SortOutput { keys, values }
}

async fn check(input: &SortFixture, bits: SortKeyWidth) -> Result<()> {
    let output = sorted(input, bits).await?;
    let expected = reference(input);
    assert_eq!(output.keys, expected.keys, "keys are not in order");
    assert_eq!(
        output.values, expected.values,
        "payloads did not travel with their keys (or the sort is not stable)"
    );
    Ok(())
}

#[tokio::test]
async fn sorts_a_single_partial_tile() -> Result<()> {
    check(&SortFixture::generated(37.into(), 7.into()), 32.into()).await
}

#[tokio::test]
async fn sorts_exactly_one_tile() -> Result<()> {
    check(&SortFixture::generated(256.into(), 11.into()), 32.into()).await
}

#[tokio::test]
async fn sorts_many_tiles() -> Result<()> {
    check(&SortFixture::generated(50_000.into(), 13.into()), 32.into()).await
}

/// The count is deliberately not a multiple of the tile size, so the last tile
/// is partly out of range -- the case where an inactive lane could corrupt a
/// live lane's rank.
#[tokio::test]
async fn sorts_a_ragged_tail() -> Result<()> {
    check(&SortFixture::generated(4097.into(), 17.into()), 32.into()).await
}

/// Every key equal exercises the widest possible rank counting: one digit
/// holds a whole tile, every pass.
#[tokio::test]
async fn sorts_all_equal_keys() -> Result<()> {
    check(&SortFixture::from(vec![42u32; 1000].as_slice()), 32.into()).await
}

/// Already sorted, and its reverse: the two orderings a stable sort is most
/// likely to get wrong in opposite directions.
#[tokio::test]
async fn sorts_presorted_and_reversed() -> Result<()> {
    let ascending: Vec<u32> = (0..3000).collect();
    check(&SortFixture::from(ascending.as_slice()), 32.into()).await?;
    let descending: Vec<u32> = (0..3000).rev().collect();
    check(&SortFixture::from(descending.as_slice()), 32.into()).await
}

/// 30-bit Morton codes take four passes but must not disturb the top two bits,
/// and the caller is entitled to say so with `bits`.
#[tokio::test]
async fn sorts_thirty_bit_keys() -> Result<()> {
    let mut input = SortFixture::generated(10_000.into(), 19.into());
    for key in &mut input.0 {
        key.0 &= 0x3FFF_FFFF;
    }
    check(&input, 30.into()).await
}

/// A key that varies only in its high byte still has to move: the low passes
/// are no-ops that must leave the order alone for the last one to fix.
#[tokio::test]
async fn sorts_on_the_high_byte_alone() -> Result<()> {
    let mut input = SortFixture::generated(2000.into(), 23.into());
    for key in &mut input.0 {
        key.0 &= 0xFF00_0000;
    }
    check(&input, 32.into()).await
}

/// Fewer than two elements is a no-op, not an error. (Zero is checked through
/// `record` rather than `check`, because a zero-length buffer cannot be
/// allocated in the first place.)
#[tokio::test]
async fn accepts_degenerate_counts() -> Result<()> {
    check(&SortFixture::from([7u32].as_slice()), 32.into()).await?;

    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;
    let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
    let buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0u32; 4]),
        definition,
    )?;
    let mut scratch = SortScratch::new(&context, 4.into())?;
    let mut batch = KernelBatch::new(&context);
    sort.record(
        &mut batch,
        &buffer,
        &buffer,
        &mut scratch,
        0.into(),
        32.into(),
    )?;
    assert_eq!(
        batch.dispatch_count(),
        crate::RecordedDispatchCount::default(),
        "an empty sort must dispatch nothing"
    );
    Ok(())
}

#[tokio::test]
async fn rejects_a_scratch_that_is_too_small() -> Result<()> {
    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;
    let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
    let data: Vec<u32> = (0..1000).collect();
    let key_buffer = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&data),
        definition.clone(),
    )?;
    let value_buffer =
        Buffer::from_upload(&context, BufferUpload::from_elements(&data), definition)?;
    let mut scratch = SortScratch::new(&context, 100.into())?;

    let mut batch = KernelBatch::new(&context);
    let result = sort.record(
        &mut batch,
        &key_buffer,
        &value_buffer,
        &mut scratch,
        1000.into(),
        32.into(),
    );
    assert!(
        result.is_err(),
        "a 100-element scratch must not accept 1000"
    );
    Ok(())
}

#[test]
fn pass_count_rounds_up_to_whole_digits() {
    assert_eq!(SortKeyWidth::from(1).digits().count(), 1);
    assert_eq!(SortKeyWidth::from(8).digits().count(), 1);
    assert_eq!(SortKeyWidth::from(9).digits().count(), 2);
    assert_eq!(SortKeyWidth::from(16).digits().count(), 2);
    assert_eq!(SortKeyWidth::from(30).digits().count(), 4);
    assert_eq!(SortKeyWidth::from(32).digits().count(), 4);
}

#[tokio::test]
async fn preserves_original_gpu_words_and_shader_source() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/radix_sort.json")).unwrap();
    let shaders = [histogram_code(), scan_code(), scatter_code()];
    for (actual, expected) in shaders.iter().zip(fixture["shaders"].as_array().unwrap()) {
        assert_eq!(<&str>::from(actual), expected.as_str().unwrap());
    }
    for case in fixture["widths"].as_array().unwrap() {
        let width = SortKeyWidth::from(case["bits"].as_u64().unwrap() as u32);
        assert_eq!(
            width.digits().count(),
            case["passes"].as_u64().unwrap() as usize
        );
        assert_eq!(width.pass_count().to_string(), case["passes"].to_string());
    }
    for case in fixture["layout"].as_array().unwrap() {
        let count = SortItemCount::from(case["count"].as_u64().unwrap() as u32);
        assert_eq!(
            u64::from(count.word_bytes()),
            case["bytes"].as_u64().unwrap()
        );
        assert_eq!(
            count.with_sentinel().to_string(),
            case["capacity"].to_string()
        );
        assert_eq!(
            u64::from(count.with_sentinel().tiles().histogram_bytes()),
            case["histogram_bytes"].as_u64().unwrap()
        );
    }
    let context = WgpuContext::new().await.unwrap();
    let sort = RadixSort::new(&context).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let count = SortItemCount::from(case["count"].as_u64().unwrap() as u32);
        let width = SortKeyWidth::from(case["bits"].as_u64().unwrap() as u32);
        let mut input = Vec::new();
        for word in case["input"].as_array().unwrap() {
            input.push(FixtureKey(word.as_u64().unwrap() as u32));
        }
        let values: Vec<FixturePayload> = (0..input.len() as u32).map(FixturePayload).collect();
        let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
        let keys = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&input),
            definition.clone(),
        )
        .unwrap();
        let payloads =
            Buffer::from_upload(&context, BufferUpload::from_elements(&values), definition)
                .unwrap();
        let mut scratch = SortScratch::new(&context, count).unwrap();
        sort.run(&context, &keys, &payloads, &mut scratch, count, width)
            .unwrap();
        let mut expected_keys = Vec::new();
        for word in case["keys"].as_array().unwrap() {
            expected_keys.push(FixtureKey(word.as_u64().unwrap() as u32));
        }
        let mut expected_values = Vec::new();
        for word in case["values"].as_array().unwrap() {
            expected_values.push(FixturePayload(word.as_u64().unwrap() as u32));
        }
        assert_eq!(
            keys.read::<FixtureKey>(&context).await.unwrap(),
            expected_keys
        );
        assert_eq!(
            payloads.read::<FixturePayload>(&context).await.unwrap(),
            expected_values
        );
    }
}

#[tokio::test]
async fn admission_priority_growth_and_stage_errors_remain_classifiable() {
    use std::error::Error;
    let context = WgpuContext::new().await.unwrap();
    let mut sort = RadixSort::new(&context).unwrap();
    let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
    let small = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0u32; 2]),
        definition.clone(),
    )
    .unwrap();
    let large = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0u32; 4]),
        definition,
    )
    .unwrap();
    let mut scratch = SortScratch::new(&context, 0.into()).unwrap();
    assert_eq!(scratch.capacity(), SortItemCount::from(1));
    assert_eq!(
        scratch.ensure(&context, 1.into()).unwrap(),
        ScratchGrowth::Unchanged
    );
    assert_eq!(
        scratch.ensure(&context, 2.into()).unwrap(),
        ScratchGrowth::Grown
    );
    let mut batch = KernelBatch::new(&context);
    let error = sort
        .record(
            &mut batch,
            &small,
            &small,
            &mut scratch,
            4.into(),
            32.into(),
        )
        .unwrap_err();
    assert!(
        matches!(&error,SortError::ScratchCapacity {capacity,count} if *capacity==SortItemCount::from(2)&&*count==SortItemCount::from(4))
    );
    assert!(error.source().is_none());
    assert_eq!(
        error.to_string(),
        "RadixSort: scratch holds 2 elements, asked to sort 4"
    );
    scratch.ensure(&context, 4.into()).unwrap();
    let error = sort
        .record(
            &mut batch,
            &small,
            &large,
            &mut scratch,
            4.into(),
            32.into(),
        )
        .unwrap_err();
    assert!(
        matches!(&error,SortError::BufferLength {count,needed,keys,values} if *count==SortItemCount::from(4)&&*needed==BufferByteLength::from(16u64)&&*keys==BufferByteLength::from(8u64)&&*values==BufferByteLength::from(16u64))
    );
    assert_eq!(
        error.to_string(),
        "RadixSort: 4 elements need 16 bytes; keys hold 8, values hold 16"
    );
    assert!(error.source().is_none());
    // An existing sorter kernel expects scatter resources absent from the
    // histogram stage. This real dispatch must retain its digit and cause.
    sort.histogram = sort.scatter.clone();
    let error = sort
        .record(
            &mut batch,
            &large,
            &large,
            &mut scratch,
            4.into(),
            32.into(),
        )
        .unwrap_err();
    assert!(
        matches!(&error,SortError::Dispatch {stage,..} if stage.kernel==SortKernelRole::Histogram&&stage.digit==SortKeyWidth::from(32).digits().next().unwrap())
    );
    assert!(error.source().unwrap().is::<KernelDispatchError>());
    if let SortError::Dispatch { source, .. } = &error {
        assert_eq!(error.to_string(), source.to_string());
    }
}
