use super::*;
use std::error::Error;

#[tokio::test]
async fn record_mismatches_keep_nominal_counts_and_precede_kernel_creation() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let name = ConstraintName::from("rejected");
    let edges = ConstraintEdges::from([[0u32, 1]].as_slice());
    let error = ConstraintSet::distance(
        &context,
        &cache,
        name.clone(),
        &edges,
        &[1.0, 2.0],
        (0.0).into(),
    )
    .err()
    .unwrap();
    assert!(
        matches!(&error,ConstraintBuildError::DistanceRecordCount {set,edges,rest_lengths} if *set==name && *edges==ConstraintCount::from(1) && *rest_lengths==ConstraintCount::from(2))
    );
    assert!(error.source().is_none());
    assert_eq!(
        error.to_string(),
        "ConstraintSet::distance: 1 edges but 2 rest lengths"
    );
    let error = ConstraintSet::spring(
        &context,
        &cache,
        name.clone(),
        &edges,
        &[1.0],
        &[],
        (0.0).into(),
    )
    .err()
    .unwrap();
    assert!(
        matches!(&error,ConstraintBuildError::SpringRecordCount {set,edges,rest_lengths,spring_params} if *set==name && *edges==ConstraintCount::from(1) && *rest_lengths==ConstraintCount::from(1) && *spring_params==ConstraintCount::from(0))
    );
    assert_eq!(
        error.to_string(),
        "ConstraintSet::spring: 1 edges, 1 rest lengths, 0 spring params"
    );
    assert!(error.source().is_none());
}

#[tokio::test]
async fn dispatch_error_keeps_set_color_and_concrete_provider_cause() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let kernel = cache
        .get(&context, &wgsl::constraint_kernel(wgsl::DISTANCE))
        .unwrap();
    let incidence = ConstraintIncidence::from_native_records(&[[0u32, 1]]).unwrap();
    // Generic construction deliberately leaves the distance kernel's rest
    // attachment absent; the real dispatch must classify that missing binding.
    let set = ConstraintSet::new(
        &context,
        "missing rest".into(),
        kernel,
        &cache,
        &incidence,
        (0.0).into(),
    )
    .unwrap();
    assert_eq!(set.constraint_count(), ConstraintCount::from(1));
    assert_eq!(set.arity(), ConstraintArity::try_from(2).unwrap());
    let particles = Particles::from_positions(
        &context,
        &[fabelgeist_math::Vec3::default(); 2],
        &[1.0.into(); 2],
    )
    .unwrap();
    let mut batch = KernelBatch::new(&context);
    let error = set
        .record_solve(&mut batch, &particles, (1.0 / 60.0).into())
        .unwrap_err();
    assert_eq!(error.set, ConstraintName::from("missing rest"));
    assert_eq!(
        error.stage,
        ConstraintDispatchStage::Solve(set.coloring().colors().next().unwrap())
    );
    assert!(error.source().unwrap().is::<KernelDispatchError>());
    assert!(
        matches!(error.source.as_ref(),KernelDispatchError::Buffer {name,..} if *name==ShaderBindingName::from("rest_lengths"))
    );
    assert_eq!(error.to_string(), error.source.to_string());
}

#[test]
fn construction_errors_keep_allocation_and_cache_causes() {
    let name = ConstraintName::from("failed set");
    let error = ConstraintBuildError::Allocation {
        set: name.clone(),
        buffer: ConstraintBuffer::Lambdas,
        source: BufferCreationError::Empty,
    };
    assert!(error.source().unwrap().is::<BufferCreationError>());
    assert!(
        matches!(&error,ConstraintBuildError::Allocation {set,buffer,..} if *set==name && *buffer==ConstraintBuffer::Lambdas)
    );
    assert_eq!(error.to_string(), "Buffer size must be greater than 0");
    let error = ConstraintBuildError::Kernel {
        set: name,
        kernel: ConstraintKernel::Clear,
        source: KernelCacheError::ReadPoisoned,
    };
    assert!(error.source().unwrap().is::<KernelCacheError>());
    assert_eq!(error.to_string(), "Failed to read kernel cache");
}
