use super::*;
use serde_json::json;

fn rejected(result: anyhow::Result<ConstraintSet>) -> String {
    match result {
        Ok(_) => panic!("invalid fixture admitted"),
        Err(error) => error.to_string(),
    }
}

#[tokio::test]
async fn preserves_native_upload_allocation_admission_and_recording() -> anyhow::Result<()> {
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/constraint-native.json"))?;
    let context = WgpuContext::new().await?;
    let cache = KernelCache::new();
    let mut messages = vec![
        ConstraintArity::try_from(0).unwrap_err().to_string(),
        ConstraintIncidence::from_native_flat(&[0, 1, 2], ConstraintArity::try_from(2)?)
            .unwrap_err()
            .to_string(),
    ];
    messages.push(rejected(ConstraintSet::distance(
        &context,
        &cache,
        "distance",
        &ConstraintEdges::from([[0, 1]].as_slice()),
        &[1.0, 2.0],
        0.0,
    )));
    messages.push(rejected(ConstraintSet::spring(
        &context,
        &cache,
        "spring",
        &ConstraintEdges::from([[0, 1]].as_slice()),
        &[1.0, 2.0],
        &[],
        0.0,
    )));
    let native = [[0, 1], [1, 2], [2, 3], [3, 0], [u32::MAX, 7]];
    let mut set = ConstraintSet::distance(
        &context,
        &cache,
        "distance",
        &ConstraintEdges::from(native.as_slice()),
        &[0.1, 0.2, 0.3, 0.4, 0.5],
        0.0,
    )?;
    let particle_words = set.particles.read::<u32>(&context).await?;
    let rest_words = set.attachments[0].1.read::<u32>(&context).await?;
    let reordered = set.reorder(&[5u32, 6, 7, 8, 9]);
    let lambda_bytes = u64::from(set.lambdas.size);
    let mut live_count = KernelBatch::labelled(&context, "live");
    set.record_clear(&mut live_count)?;
    let clear_dispatches = live_count.dispatch_count();
    set.enabled = false;
    let mut disabled = KernelBatch::labelled(&context, "disabled");
    set.record_clear(&mut disabled)?;
    let disabled_dispatches = disabled.dispatch_count();
    let mut empty = ConstraintSet::distance(
        &context,
        &cache,
        "empty",
        &ConstraintEdges::default(),
        &[],
        0.0,
    )?;
    let empty_words = empty.particles.read::<u32>(&context).await?;
    let empty_attachment_words = empty.attachments[0].1.read::<u32>(&context).await?;
    let held = empty.attachments[0].1.clone();
    messages.push(
        empty
            .attach(&context, "rest_lengths", &[1u32])
            .unwrap_err()
            .to_string(),
    );
    assert_eq!(empty.attachments.len(), 1);
    assert_eq!(empty.attachments[0].1, held);
    assert_eq!(
        empty.attachments[0].1.read::<u32>(&context).await?,
        empty_attachment_words
    );
    let held = set.attachments[0].1.clone();
    messages.push(
        set.attach_raw(&context, "weights", &[1u32])
            .unwrap_err()
            .to_string(),
    );
    assert_eq!(set.attachments.len(), 1);
    assert_eq!(set.attachments[0].1, held);
    assert_eq!(
        set.attachments[0].1.read::<u32>(&context).await?,
        rest_words
    );
    let mut empty_batch = KernelBatch::labelled(&context, "empty");
    empty.record_clear(&mut empty_batch)?;
    let actual = json!({"messages":messages,"particle_words":particle_words,
        "rest_words":rest_words,"reordered":reordered,"lambda_bytes":lambda_bytes,
        "empty_words":empty_words,"empty_attachment_words":empty_attachment_words,
        "empty_lambda_bytes":u64::from(empty.lambdas.size),
        "clear_dispatches":clear_dispatches,"disabled_dispatches":disabled_dispatches,
        "empty_dispatches":empty_batch.dispatch_count()});
    assert_eq!(actual, expected);
    // An empty set keeps the existing raw-attachment admission policy.
    empty.attach_raw(&context, "raw", &[7u32, 8])?;
    assert_eq!(empty.attachments[1].1.read::<u32>(&context).await?, [7, 8]);
    Ok(())
}
