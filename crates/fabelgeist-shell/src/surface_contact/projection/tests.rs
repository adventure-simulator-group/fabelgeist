//! Failure classification and admission preserve the pre-projection state.
use super::*;
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, BufferUse, ReadbackError};
use fabelgeist_xpbd::{ParticleBufferRole, ParticleCount, ParticleError, ParticleInputCount};
use std::error::Error;

#[tokio::test]
async fn interval_mismatch_retains_counts_and_does_not_mutate_state() {
    let context = fabelgeist_gpu::globals::WgpuContext::new()
        .await
        .expect("GPU context");
    let positions = [Vec3::new(0.0, 1.0, 0.0)];
    let particles = fabelgeist_xpbd::Particles::from_positions(&context, &positions, &[1.0.into()])
        .expect("particle state");
    let contacts = SurfaceContacts::new(1, vec![]);
    let before = particles.read_positions(&context).await.expect("positions");
    let velocities = particles
        .read_velocities(&context)
        .await
        .expect("velocities");
    let error = contacts
        .project_particles(&context, &particles, 0.005, 1, Some(&[]))
        .await
        .expect_err("short interval must fail admission");
    assert!(matches!(
        error,
        SurfaceProjectionError::IntervalCount { expected, actual }
            if expected == ParticleCount::from(1) && actual == ParticleInputCount::from(0)
    ));
    assert!(error.source().is_none());
    assert_eq!(
        error.to_string(),
        "interval start does not match the particle count"
    );
    assert_eq!(
        particles.read_positions(&context).await.expect("positions"),
        before
    );
    assert_eq!(
        particles
            .read_velocities(&context)
            .await
            .expect("velocities"),
        velocities
    );
}

#[tokio::test]
async fn projection_readback_errors_retain_stage_and_concrete_cause() {
    let context = fabelgeist_gpu::globals::WgpuContext::new()
        .await
        .expect("GPU context");
    let positions = [Vec3::new(0.0, 1.0, 0.0)];
    let mut particles =
        fabelgeist_xpbd::Particles::from_positions(&context, &positions, &[1.0.into()])
            .expect("particle state");
    let malformed = Buffer::new(
        &context,
        20u64.into(),
        BufferDefinition::storage().with_usage(BufferUse::CopySource),
    )
    .expect("partial-record buffer");
    let previous = std::mem::replace(&mut particles.previous, malformed.clone());
    let contacts = SurfaceContacts::new(1, vec![]);
    let error = contacts
        .project_particles(&context, &particles, 0.005, 1, None)
        .await
        .expect_err("previous records must be complete");
    assert!(matches!(
        &error,
        SurfaceProjectionError::PreviousReadback { source }
            if matches!(source.as_ref(), ReadbackError::PartialElement { length, element_bytes }
                if *length == 20u64.into() && *element_bytes == 16u64.into())
    ));
    assert!(
        error
            .source()
            .expect("provider cause")
            .is::<ReadbackError>()
    );
    particles.previous = previous;

    let saved_positions = std::mem::replace(&mut particles.positions, malformed.clone());
    let error = contacts
        .project_particles(&context, &particles, 0.005, 1, Some(&positions))
        .await
        .expect_err("position records must be complete");
    assert!(matches!(
        &error,
        SurfaceProjectionError::ParticleState {
            stage: SurfaceProjectionStage::PositionsRead,
            source: ParticleError::Readback { role: ParticleBufferRole::Positions, source },
        } if matches!(source.as_ref(), ReadbackError::PartialElement { .. })
    ));
    assert!(
        error
            .source()
            .expect("particle cause")
            .is::<ParticleError>()
    );
    assert!(
        error
            .source()
            .expect("particle cause")
            .source()
            .expect("provider cause")
            .is::<ReadbackError>()
    );
    particles.positions = saved_positions;

    particles.velocities = malformed;
    let error = contacts
        .project_particles(&context, &particles, 0.005, 1, Some(&positions))
        .await
        .expect_err("velocity records must be complete");
    assert!(matches!(
        &error,
        SurfaceProjectionError::ParticleState {
            stage: SurfaceProjectionStage::VelocitiesRead,
            source: ParticleError::Readback { role: ParticleBufferRole::Velocities, source },
        } if matches!(source.as_ref(), ReadbackError::PartialElement { .. })
    ));
}

#[tokio::test]
async fn outer_layer_preserves_the_particle_error_directly() {
    let context = fabelgeist_gpu::globals::WgpuContext::new()
        .await
        .expect("GPU context");
    let mut particles =
        fabelgeist_xpbd::Particles::from_positions(&context, &[Vec3::default()], &[1.0.into()])
            .expect("particle state");
    particles.positions = Buffer::new(
        &context,
        20u64.into(),
        BufferDefinition::storage().with_usage(BufferUse::CopySource),
    )
    .expect("partial-record buffer");
    let layer = crate::outer_layer::OuterLayer::new(
        vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
        ],
        vec![[0, 1, 2]],
        0.003,
        Vec3::new(0.0, 0.0, -1.0),
    );
    let error: ParticleError = layer
        .project_particles(&context, &particles, &[])
        .await
        .expect_err("particle cause must survive projection");
    assert!(matches!(
        error,
        ParticleError::Readback {
            role: ParticleBufferRole::Positions,
            ..
        }
    ));
}
