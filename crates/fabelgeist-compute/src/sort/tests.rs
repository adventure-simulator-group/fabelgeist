use super::*;

/// Deterministic pseudo-random keys. A fixed generator rather than a crate, so
/// that a failure is reproducible from the test name alone.
fn keys(count: u32, seed: u32) -> Vec<u32> {
    let mut state = seed | 1;
    (0..count)
        .map(|_| {
            // xorshift32
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        })
        .collect()
}

async fn sorted(keys: &[u32], bits: u32) -> Result<(Vec<u32>, Vec<u32>)> {
    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;

    let count = keys.len() as u32;
    // The payload is each key's original index, so the assertions can check
    // that values travelled with their keys.
    let values: Vec<u32> = (0..count).collect();

    let definition = BufferDefinition::storage().with_copy_src();
    let key_buffer = Buffer::from_slice(&context, keys, definition.clone())?;
    let value_buffer = Buffer::from_slice(&context, &values, definition)?;
    let mut scratch = SortScratch::new(&context, count)?;

    sort.run(
        &context,
        &key_buffer,
        &value_buffer,
        &mut scratch,
        count,
        bits,
    )?;

    Ok((
        key_buffer.read(&context).await?,
        value_buffer.read(&context).await?,
    ))
}

/// What the GPU should have produced: a stable sort by key.
fn reference(input: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let mut pairs: Vec<(u32, u32)> = input.iter().copied().zip(0u32..).collect();
    pairs.sort_by_key(|&(key, _)| key);
    pairs.into_iter().unzip()
}

async fn check(input: &[u32], bits: u32) -> Result<()> {
    let (keys_out, values_out) = sorted(input, bits).await?;
    let (expected_keys, expected_values) = reference(input);
    assert_eq!(keys_out, expected_keys, "keys are not in order");
    assert_eq!(
        values_out, expected_values,
        "payloads did not travel with their keys (or the sort is not stable)"
    );
    Ok(())
}

#[tokio::test]
async fn sorts_a_single_partial_tile() -> Result<()> {
    check(&keys(37, 7), 32).await
}

#[tokio::test]
async fn sorts_exactly_one_tile() -> Result<()> {
    check(&keys(256, 11), 32).await
}

#[tokio::test]
async fn sorts_many_tiles() -> Result<()> {
    check(&keys(50_000, 13), 32).await
}

/// The count is deliberately not a multiple of the tile size, so the last tile
/// is partly out of range -- the case where an inactive lane could corrupt a
/// live lane's rank.
#[tokio::test]
async fn sorts_a_ragged_tail() -> Result<()> {
    check(&keys(4097, 17), 32).await
}

/// Every key equal exercises the widest possible rank counting: one digit
/// holds a whole tile, every pass.
#[tokio::test]
async fn sorts_all_equal_keys() -> Result<()> {
    check(&vec![42u32; 1000], 32).await
}

/// Already sorted, and its reverse: the two orderings a stable sort is most
/// likely to get wrong in opposite directions.
#[tokio::test]
async fn sorts_presorted_and_reversed() -> Result<()> {
    let ascending: Vec<u32> = (0..3000).collect();
    check(&ascending, 32).await?;
    let descending: Vec<u32> = (0..3000).rev().collect();
    check(&descending, 32).await
}

/// 30-bit Morton codes take four passes but must not disturb the top two bits,
/// and the caller is entitled to say so with `bits`.
#[tokio::test]
async fn sorts_thirty_bit_keys() -> Result<()> {
    let input: Vec<u32> = keys(10_000, 19)
        .into_iter()
        .map(|k| k & 0x3FFF_FFFF)
        .collect();
    check(&input, 30).await
}

/// A key that varies only in its high byte still has to move: the low passes
/// are no-ops that must leave the order alone for the last one to fix.
#[tokio::test]
async fn sorts_on_the_high_byte_alone() -> Result<()> {
    let input: Vec<u32> = keys(2000, 23)
        .into_iter()
        .map(|k| k & 0xFF00_0000)
        .collect();
    check(&input, 32).await
}

/// Fewer than two elements is a no-op, not an error. (Zero is checked through
/// `record` rather than `check`, because a zero-length buffer cannot be
/// allocated in the first place.)
#[tokio::test]
async fn accepts_degenerate_counts() -> Result<()> {
    check(&[7], 32).await?;

    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;
    let definition = BufferDefinition::storage().with_copy_src();
    let buffer = Buffer::from_slice(&context, &[0u32; 4], definition)?;
    let mut scratch = SortScratch::new(&context, 4)?;
    let mut batch = KernelBatch::new(&context);
    sort.record(&mut batch, &buffer, &buffer, &mut scratch, 0, 32)?;
    assert_eq!(
        batch.dispatch_count(),
        0,
        "an empty sort must dispatch nothing"
    );
    Ok(())
}

#[tokio::test]
async fn rejects_a_scratch_that_is_too_small() -> Result<()> {
    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;
    let definition = BufferDefinition::storage().with_copy_src();
    let data: Vec<u32> = (0..1000).collect();
    let key_buffer = Buffer::from_slice(&context, &data, definition.clone())?;
    let value_buffer = Buffer::from_slice(&context, &data, definition)?;
    let mut scratch = SortScratch::new(&context, 100)?;

    let mut batch = KernelBatch::new(&context);
    let result = sort.record(
        &mut batch,
        &key_buffer,
        &value_buffer,
        &mut scratch,
        1000,
        32,
    );
    assert!(
        result.is_err(),
        "a 100-element scratch must not accept 1000"
    );
    Ok(())
}

#[test]
fn pass_count_rounds_up_to_whole_digits() {
    assert_eq!(RadixSort::passes_for(1), 1);
    assert_eq!(RadixSort::passes_for(8), 1);
    assert_eq!(RadixSort::passes_for(9), 2);
    assert_eq!(RadixSort::passes_for(16), 2);
    assert_eq!(RadixSort::passes_for(30), 4);
    assert_eq!(RadixSort::passes_for(32), 4);
}
