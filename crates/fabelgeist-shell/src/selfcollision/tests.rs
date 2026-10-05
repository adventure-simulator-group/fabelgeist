use super::*;

#[test]
fn host_adjacency_preserves_empty_slots_order_and_repeated_edges() {
    assert_eq!(
        SelfCollision::adjacency(4usize.into(), &[[2, 1], [0, 1], [2, 1]]),
        vec![vec![1], vec![2, 0, 2], vec![1, 1], vec![]]
    );
}

#[tokio::test]
async fn count_admission_keeps_empty_noops_and_rejected_batches_unmodified() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let error = SelfCollision::new(&context, &cache, 1.into(), &[], 0.01)
        .err()
        .unwrap();
    assert_eq!(
        error.to_string(),
        "SelfCollision: 1 particles but 0 adjacency lists"
    );
    let mut collision = SelfCollision::new(&context, &cache, 0.into(), &[], 0.01).unwrap();
    let empty = Particles::new(&context, 0.into()).unwrap();
    let mut batch = KernelBatch::labelled(&context, "particle admission");
    collision.record(&mut batch, &empty, true).unwrap();
    assert_eq!(batch.dispatch_count(), 0);
    let two = Particles::from_positions(
        &context,
        &[fabelgeist_math::Vec3::default(); 2],
        &[fabelgeist_xpbd::ParticleInverseMass::PINNED; 2],
    )
    .unwrap();
    let error = collision.record(&mut batch, &two, true).unwrap_err();
    assert_eq!(
        error.to_string(),
        "SelfCollision: built for 1 particles, given 2"
    );
    assert_eq!(batch.dispatch_count(), 0);
}
