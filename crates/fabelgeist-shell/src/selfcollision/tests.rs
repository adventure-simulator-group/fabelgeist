use super::*;
use crate::native_vector_fixture::{decode_native_vectors, encode_native_vectors};
use fabelgeist_compute::{KernelCache, KernelDispatchError, SortError, SortScratch};
use fabelgeist_gpu::prelude::BufferCreationError;
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::ParticleInverseMass;
use std::error::Error;

#[tokio::test]
async fn adjacency_admission_precedes_kernel_compilation() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let error = SelfCollision::new(&context, &cache, 2.into(), &[vec![]], 0.005)
        .err()
        .expect("adjacency mismatch");
    assert!(
        matches!(error, SelfCollisionBuildError::Adjacency { particles, lists } if particles == ParticleCount::from(2) && lists == ParticleInputCount::from(1))
    );
    assert_eq!(
        error.to_string(),
        "SelfCollision: 2 particles but 1 adjacency lists"
    );
    assert!(error.source().is_none());
    assert_eq!(cache.count().unwrap().to_string(), "0");
}

#[tokio::test]
async fn disabled_and_small_passes_precede_capacity_admission() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let mut collision =
        SelfCollision::new(&context, &cache, 2.into(), &[vec![], vec![]], 0.005).unwrap();
    let mut particles = Particles::from_positions(
        &context,
        &[Vec3::default(); 3],
        &[ParticleInverseMass::UNIT_MASS; 3],
    )
    .unwrap();
    let mut batch = KernelBatch::labelled(&context, "capacity admission".into());
    collision.enabled = false;
    collision.record(&mut batch, &particles, false).unwrap();
    collision.enabled = true;
    let error = collision.record(&mut batch, &particles, false).unwrap_err();
    assert!(
        matches!(error, SelfCollisionRecordError::Capacity { capacity, count } if capacity == ParticleCapacity::from(2) && count == ParticleCount::from(3))
    );
    assert_eq!(
        error.to_string(),
        "SelfCollision: built for 2 particles, given 3"
    );
    assert!(error.source().is_none());
    particles
        .write(
            &context,
            &[Vec3::default()],
            &[ParticleInverseMass::UNIT_MASS],
        )
        .unwrap();
    collision.record(&mut batch, &particles, false).unwrap();
    assert!(!collision.built);
}

#[tokio::test]
async fn allocation_failure_retains_its_buffer_and_native_cause() {
    let context = WgpuContext::new().await.expect("device context");
    let error = SelfCollisionBuffer::Corrections
        .allocate(&context, 0u64.into(), BufferDefinition::storage())
        .expect_err("zero allocation");
    assert!(matches!(
        error,
        SelfCollisionBuildError::Allocation {
            buffer: SelfCollisionBuffer::Corrections,
            source: BufferCreationError::Empty
        }
    ));
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<BufferCreationError>(),
        Some(&BufferCreationError::Empty)
    );
    assert_eq!(error.to_string(), "Buffer size must be greater than 0");
}

#[tokio::test]
async fn dispatch_failure_retains_the_kernel_and_binding_cause() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let kernel = SelfCollisionKernel::Hash.load(&context, &cache).unwrap();
    let mut batch = KernelBatch::labelled(&context, "dispatch admission".into());
    let error = SelfCollisionKernel::Hash
        .dispatch(&mut batch, &kernel, &PassParameters::new(), 1.into())
        .unwrap_err();
    assert!(matches!(
        error,
        SelfCollisionRecordError::Dispatch {
            kernel: SelfCollisionKernel::Hash,
            ..
        }
    ));
    assert!(
        matches!(error.source().unwrap().downcast_ref::<KernelDispatchError>(), Some(KernelDispatchError::Buffer { name, .. }) if *name == ShaderBindingName::from("positions"))
    );
}

#[tokio::test]
async fn sort_failure_keeps_capacity_and_stage_below_the_collision_error() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let mut collision =
        SelfCollision::new(&context, &cache, 2.into(), &[vec![], vec![]], 0.005).unwrap();
    collision.scratch = SortScratch::new(&context, 0.into()).unwrap();
    let particles = Particles::from_positions(
        &context,
        &[Vec3::default(); 2],
        &[ParticleInverseMass::UNIT_MASS; 2],
    )
    .unwrap();
    let mut batch = KernelBatch::labelled(&context, "sort admission".into());
    let error = collision.record(&mut batch, &particles, true).unwrap_err();
    assert!(
        matches!(error, SelfCollisionRecordError::Sort(SortError::ScratchCapacity { capacity, count }) if capacity == 1.into() && count == 2.into())
    );
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<SortError>()
            .is_some()
    );
}

#[tokio::test]
async fn preserves_original_hash_rebuild_reuse_and_pinned_gpu_words() {
    let context = WgpuContext::new().await.expect("device context");
    let cache = KernelCache::new();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/self_collision.json"))
            .expect("original native GPU states");
    let cases = fixture["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 7);
    for case in cases {
        let positions = decode_native_vectors(&case["initial"]);
        let mut masses = Vec::new();
        for word in case["inverse_mass_words"].as_array().unwrap() {
            masses.push(ParticleInverseMass::from(f32::from_bits(
                word.as_u64().unwrap() as u32,
            )));
        }
        let mut adjacency = Vec::new();
        for list in case["adjacency"].as_array().unwrap() {
            let mut neighbors = Vec::new();
            for word in list.as_array().unwrap() {
                neighbors.push(word.as_u64().unwrap() as u32);
            }
            adjacency.push(neighbors);
        }
        let particles = Particles::from_positions(&context, &positions, &masses).unwrap();
        let mut collision = SelfCollision::new(
            &context,
            &cache,
            particles.count(),
            &adjacency,
            f32::from_bits(case["radius_word"].as_u64().unwrap() as u32),
        )
        .unwrap();
        collision.enabled = case["enabled"].as_bool().unwrap();
        let states = case["states"].as_array().unwrap();
        assert_eq!(states.len(), 3);
        for state in states {
            let mut batch = KernelBatch::labelled(&context, "self-collision native fixture".into());
            collision
                .record(&mut batch, &particles, state["rebuild"].as_bool().unwrap())
                .unwrap();
            batch.submit();
            assert_eq!(
                encode_native_vectors(&particles.read_positions(&context).await.unwrap()),
                state["positions"]
            );
            assert_eq!(
                encode_native_vectors(&particles.read_velocities(&context).await.unwrap()),
                state["velocities"]
            );
            let words: Vec<_> = particles
                .read_inverse_masses(&context)
                .await
                .unwrap()
                .into_iter()
                .map(u32::from)
                .collect();
            assert_eq!(serde_json::json!(words), state["mass_words"]);
        }
    }
}
