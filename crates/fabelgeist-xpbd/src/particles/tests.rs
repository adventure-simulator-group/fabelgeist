use super::*;

/// Native JSON word snapshots are decoded only at the fixture comparison.
struct ParticleSnapshot(serde_json::Value);
impl ParticleSnapshot {
    async fn capture(particles: &Particles, context: &WgpuContext) -> Self {
        let positions: Vec<f32> = particles.positions.read(context).await.unwrap();
        let previous: Vec<f32> = particles.previous.read(context).await.unwrap();
        let velocities: Vec<f32> = particles.velocities.read(context).await.unwrap();
        let decoded = particles.read_positions(context).await.unwrap();
        let masses = particles.read_inverse_masses(context).await.unwrap();
        let mut decoded_words = Vec::new();
        for position in decoded {
            decoded_words.push([
                position.x.to_bits(),
                position.y.to_bits(),
                position.z.to_bits(),
            ]);
        }
        Self(serde_json::json!({
            "count": usize::from(particles.count()),
            "capacity": usize::from(particles.capacity().count()),
            "positions": positions.into_iter().map(f32::to_bits).collect::<Vec<_>>(),
            "previous": previous.into_iter().map(f32::to_bits).collect::<Vec<_>>(),
            "velocities": velocities.into_iter().map(f32::to_bits).collect::<Vec<_>>(),
            "decoded": decoded_words,
            "masses": masses.into_iter().map(u32::from).collect::<Vec<_>>(),
        }))
    }
}

#[tokio::test]
async fn preserves_original_particle_words_and_admission_priority() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/particles.json")).unwrap();
    let context = WgpuContext::new().await.unwrap();
    let mut positions = Vec::new();
    for position in fixture["position_words"].as_array().unwrap() {
        positions.push(Vec3::new(
            f32::from_bits(position[0].as_u64().unwrap() as u32),
            f32::from_bits(position[1].as_u64().unwrap() as u32),
            f32::from_bits(position[2].as_u64().unwrap() as u32),
        ));
    }
    let mut masses = Vec::new();
    for mass in fixture["mass_words"].as_array().unwrap() {
        masses.push(ParticleInverseMass::from(f32::from_bits(
            mass.as_u64().unwrap() as u32,
        )));
    }
    for case in fixture["cases"].as_array().unwrap() {
        let requested = ParticleCapacity::from(case["requested"].as_u64().unwrap() as u32);
        let mut particles = Particles::new(&context, requested).unwrap();
        let states = case["states"].as_array().unwrap();
        assert_eq!(
            ParticleSnapshot::capture(&particles, &context).await.0,
            states[0]
        );
        let n = usize::from(particles.capacity().count());
        particles
            .write(&context, &positions[..n], &masses[..n])
            .unwrap();
        assert_eq!(
            ParticleSnapshot::capture(&particles, &context).await.0,
            states[1]
        );
        let reversed: Vec<Vec3> = positions[..n].iter().copied().rev().collect();
        particles.write_positions(&context, &reversed).unwrap();
        assert_eq!(
            ParticleSnapshot::capture(&particles, &context).await.0,
            states[2]
        );
        particles
            .write(&context, &positions[..1], &masses[..1])
            .unwrap();
        assert_eq!(
            ParticleSnapshot::capture(&particles, &context).await.0,
            states[3]
        );
        let mismatch = particles
            .write(&context, &positions, &masses[..1])
            .unwrap_err();
        assert!(
            matches!(&mismatch, ParticleError::InputLength {positions,masses}
            if *positions == ParticleInputCount::from(4) && *masses == InverseMassCount::from(1))
        );
        let capacity = particles
            .write(&context, &[Vec3::default(); 5], &[0.0.into(); 5])
            .unwrap_err();
        assert!(
            matches!(&capacity, ParticleError::Capacity {capacity,actual}
            if *capacity == requested && *actual == ParticleInputCount::from(5))
        );
        let count = particles.write_positions(&context, &[]).unwrap_err();
        assert!(
            matches!(&count, ParticleError::PositionCount {expected,actual}
            if *expected == ParticleCount::from(1) && *actual == ParticleInputCount::from(0))
        );
        for (actual, expected) in [mismatch, capacity, count]
            .iter()
            .zip(case["errors"].as_array().unwrap())
        {
            assert_eq!(actual.to_string(), expected.as_str().unwrap());
        }
        assert_eq!(
            ParticleSnapshot::capture(&particles, &context).await.0,
            states[4]
        );
        particles.write(&context, &[], &[]).unwrap();
        assert_eq!(
            ParticleSnapshot::capture(&particles, &context).await.0,
            states[5]
        );
    }
}

#[tokio::test]
async fn readback_failure_retains_the_buffer_role_and_concrete_cause() {
    use std::error::Error;
    let context = WgpuContext::new().await.unwrap();
    let mut particles = Particles::new(&context, 1.into()).unwrap();
    particles.positions = Buffer::new(&context, 6u64.into(), BufferDefinition::storage()).unwrap();
    let error = particles.read_positions(&context).await.unwrap_err();
    assert!(
        matches!(&error, ParticleError::Readback {role: ParticleBufferRole::Positions, source}
        if matches!(source.as_ref(), ReadbackError::PartialElement {length, element_bytes}
            if *length == BufferByteLength::from(6u64) && *element_bytes == BufferByteLength::from(16u64)))
    );
    assert!(error.source().unwrap().is::<ReadbackError>());
    if let ParticleError::Readback { source, .. } = &error {
        assert_eq!(error.to_string(), source.to_string());
    }
}

#[test]
fn native_count_narrowing_and_maximum_record_extent_remain_defined() {
    let native = ParticleInputCount::from(usize::MAX).gpu_count();
    assert_eq!(native, ParticleCount::from(u32::MAX));
    assert_eq!(u64::from(native.record_bytes()), u32::MAX as u64 * 16);
    assert_eq!(
        ParticleCapacity::from(ParticleCount::EMPTY).count(),
        ParticleCount::from(1)
    );
}

#[tokio::test]
async fn complete_record_admission_rejects_partial_native_elements() {
    use std::error::Error;
    let context = WgpuContext::new().await.unwrap();
    let mut particles = Particles::new(&context, 1.into()).unwrap();
    particles.positions = Buffer::new(&context, 20u64.into(), BufferDefinition::storage()).unwrap();
    let native_words: Vec<f32> = particles.positions.read(&context).await.unwrap();
    assert_eq!(native_words.len(), 5);
    let error = particles.read_positions(&context).await.unwrap_err();
    assert!(
        matches!(&error, ParticleError::Readback {role: ParticleBufferRole::Positions, source}
        if matches!(source.as_ref(), ReadbackError::PartialElement {length, element_bytes}
            if *length == BufferByteLength::from(20u64) && *element_bytes == BufferByteLength::from(16u64)))
    );
    assert!(error.source().unwrap().is::<ReadbackError>());
    let error = ParticlePositions::new(&[Vec3::default(); 2], &[1.0.into()])
        .err()
        .unwrap();
    assert!(
        matches!(error, ParticleError::InputLength {positions, masses}
        if positions == ParticleInputCount::from(2) && masses == InverseMassCount::from(1))
    );
}

#[test]
fn velocity_upload_preserves_vector_bits_and_clears_the_unused_word() {
    let vector = Vec3::new(-0.0, f32::from_bits(0x7fc01234), f32::INFINITY);
    let records = [ParticleVelocityRecord::from(vector)];
    let words: &[u32] = bytemuck::cast_slice(&records);
    assert_eq!(words, &[0x80000000, 0x7fc01234, 0x7f800000, 0]);
}
