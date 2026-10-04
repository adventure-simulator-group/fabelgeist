use super::*;
use crate::Collider;
use fabelgeist_compute::kernel::{BufferCopyError, KernelDispatchError};
use fabelgeist_gpu::prelude::{BufferDefinition, ShaderBindingName};
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::{ParticleInverseMass, ParticlePositionRecord, ParticleVelocityRecord};
use std::error::Error;

mod scenes;
use scenes::{FixturePhase, FixtureState, SCENES};

#[tokio::test]
async fn growth_retains_allocation_and_rejected_update_leaves_state_unchanged() {
    let context = WgpuContext::new().await.unwrap();
    let mut collision = Collisions::new(&context, &KernelCache::new()).unwrap();
    assert_eq!(collision.collider_capacity, ColliderCapacity::INITIAL);
    assert_eq!(u64::from(collision.collider_buffer.length()), 1024);
    collision
        .set_colliders(&context, vec![Collider::ground(0.0); 17])
        .unwrap();
    assert_eq!(collision.collider_capacity.to_string(), "32");
    assert_eq!(u64::from(collision.collider_buffer.length()), 2048);
    collision
        .set_colliders(&context, vec![Collider::ground(1.0)])
        .unwrap();
    let before: Vec<u8> = collision.collider_buffer.read(&context).await.unwrap();
    let error = collision
        .update_colliders(&context, &[Collider::ground(2.0); 2])
        .unwrap_err();
    assert!(matches!(
        error,
        ColliderUpdateError::Count { held, provided }
            if held == ColliderCount::from(1) && provided == ColliderCount::from(2)
    ));
    assert_eq!(
        error.to_string(),
        "Collisions::update_colliders: holds 1 colliders, given 2; use `set_colliders` to change the count"
    );
    assert!(error.source().is_none());
    assert_eq!(collision.colliders(), &[Collider::ground(1.0)]);
    let after: Vec<u8> = collision.collider_buffer.read(&context).await.unwrap();
    assert_eq!(before, after);
    collision.set_colliders(&context, vec![]).unwrap();
    collision.update_colliders(&context, &[]).unwrap();
    assert_eq!(collision.collider_capacity.to_string(), "32");
    assert_eq!(u64::from(collision.collider_buffer.length()), 2048);
}

#[tokio::test]
async fn dispatch_failure_retains_analytic_or_mesh_role_and_binding_cause() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    for role in [CollisionKernel::Analytic, CollisionKernel::Mesh] {
        let kernel = role.load(&context, &cache).unwrap();
        let mut batch = KernelBatch::labelled(&context, "collision dispatch failure".into());
        let error = role
            .dispatch(&mut batch, &kernel, &PassParameters::new(), 1.into())
            .unwrap_err();
        assert!(matches!(
            error,
            CollisionRecordError::Dispatch { kernel, .. } if kernel == role
        ));
        assert!(matches!(
            error.source().unwrap().downcast_ref::<KernelDispatchError>(),
            Some(KernelDispatchError::Buffer { name, .. })
                if *name == ShaderBindingName::from("positions")
        ));
    }
}

#[tokio::test]
async fn push_out_retains_copy_failure_even_when_ordinary_collision_is_disabled() {
    let context = WgpuContext::new().await.unwrap();
    let mut collision = Collisions::new(&context, &KernelCache::new()).unwrap();
    collision.enabled = false;
    let mut particles = Particles::from_positions(
        &context,
        &[Vec3::default()],
        &[ParticleInverseMass::UNIT_MASS],
    )
    .unwrap();
    particles.previous = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0u32]),
        BufferDefinition::storage(),
    )
    .unwrap();
    let mut batch = KernelBatch::new(&context);
    collision.record(&mut batch, &particles).unwrap();
    let error = collision
        .record_push_out(&mut batch, &particles, 0.5)
        .unwrap_err();
    assert!(matches!(
        error,
        CollisionRecordError::PreviousPositionsCopy(_)
    ));
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<BufferCopyError>()
        .unwrap();
    assert_eq!(u64::from(source.bytes), 16);
    assert_eq!(u64::from(source.source_length), 16);
    assert_eq!(u64::from(source.destination_length), 4);
    assert_eq!(
        error.to_string(),
        "KernelBatch::copy_buffer: 16 bytes does not fit 16 -> 4"
    );
    particles.write(&context, &[], &[]).unwrap();
    collision
        .record_push_out(&mut batch, &particles, 0.5)
        .unwrap();
    collision.enabled = true;
    collision.record(&mut batch, &particles).unwrap();
}

#[test]
fn host_counts_retain_width_while_native_capacity_and_encoding_keep_their_contract() {
    let count = ColliderCount::from(usize::MAX);
    assert_eq!(count.to_string(), usize::MAX.to_string());
    assert_eq!(
        ColliderCapacity::INITIAL.fit(count),
        ColliderCapacityFit::GrowthRequired
    );
    #[cfg(target_pointer_width = "64")]
    {
        let wide = ColliderCount::from(u32::MAX as usize + 1);
        assert_eq!(wide.to_string(), "4294967296");
        assert_eq!(
            ColliderCapacity::INITIAL.fit(wide),
            ColliderCapacityFit::Retained
        );
        let wrapped = ColliderCapacity::for_count(wide);
        assert_eq!(wrapped.to_string(), "0");
        assert_eq!(u64::from(wrapped.byte_length()), 0);
    }
}

#[tokio::test]
async fn preserves_original_shape_mesh_growth_and_push_out_gpu_words() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let mut observed = Vec::new();
    for scene in SCENES {
        let mut state = FixtureState::new(&context, &cache, scene);
        for phase in [
            FixturePhase::Initial,
            FixturePhase::Updated,
            FixturePhase::PushOut,
        ] {
            let mut batch = KernelBatch::labelled(&context, "body collision native fixture".into());
            state.record(&context, &mut batch, phase);
            batch.submit();
            let particles = &state.particles;
            for buffer in [&particles.positions, &particles.previous] {
                let records: Vec<ParticlePositionRecord> = buffer.read(&context).await.unwrap();
                observed.extend_from_slice(bytemuck::cast_slice::<_, u8>(&records));
            }
            let records: Vec<ParticleVelocityRecord> =
                particles.velocities.read(&context).await.unwrap();
            observed.extend_from_slice(bytemuck::cast_slice::<_, u8>(&records));
        }
    }
    assert_eq!(
        observed,
        include_bytes!("../../tests/fixtures/body_collision.bin")
    );
}
