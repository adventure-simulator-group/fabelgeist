use super::*;
use fabelgeist_compute::prelude::KernelCache;
use fabelgeist_gpu::prelude::WgpuContext;
use fabelgeist_xpbd::ConstraintSet;

#[tokio::test]
async fn complete_records_survive_colour_permutation_upload_and_replacement() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let kernel = cache
        .get(
            &context,
            &fabelgeist_xpbd::wgsl::constraint_kernel(crate::wgsl::BEND),
        )
        .unwrap();
    // The first two hinges share a particle; the third joins the first colour.
    let particles = [0, 1, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10];
    let mut set =
        ConstraintSet::new(&context, "bending", kernel, &cache, &particles, 4, 0.0).unwrap();
    assert_eq!(set.coloring().order, [0, 2, 1]);
    let points = BendPoints::from([
        Vec3::default(),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.3, 0.8, 0.0),
        Vec3::new(0.7, -0.4, 0.5),
    ]);
    let weights = BendWeights::for_points(points).unwrap();
    let records = [
        weights.observed_rest(points),
        weights.flat_rest(),
        BendRecord::slack(),
    ];
    let ordered = set.reorder(&records);
    set.attach(&context, "weights", &ordered).unwrap();
    let buffer = &set
        .attachments
        .iter()
        .find(|(name, _)| name == "weights")
        .unwrap()
        .1;
    let expected_records = [records[0], records[2], records[1]];
    let expected: &[u32] = bytemuck::cast_slice(&expected_records);
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), expected);

    // A partial collection must be rejected before replacing the active binding.
    let error = set.attach(&context, "weights", &ordered[..2]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "ConstraintSet `bending`: attachment `weights` has 2 values for 3 constraints"
    );
    let buffer = &set
        .attachments
        .iter()
        .find(|(name, _)| name == "weights")
        .unwrap()
        .1;
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), expected);
    set.attach(&context, "weights", &[BendRecord::slack(); 3])
        .unwrap();
    assert_eq!(set.attachments.len(), 1);
    assert_eq!(
        set.attachments[0].1.read::<u32>(&context).await.unwrap(),
        [0; 24]
    );
}

#[tokio::test]
async fn an_empty_bending_set_preserves_its_single_zero_word_binding() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let kernel = cache
        .get(
            &context,
            &fabelgeist_xpbd::wgsl::constraint_kernel(crate::wgsl::BEND),
        )
        .unwrap();
    let mut set = ConstraintSet::new(&context, "empty bends", kernel, &cache, &[], 4, 0.0).unwrap();
    set.attach::<BendRecord>(&context, "weights", &[]).unwrap();
    assert_eq!(
        set.attachments[0].1.read::<u32>(&context).await.unwrap(),
        [0]
    );
}
