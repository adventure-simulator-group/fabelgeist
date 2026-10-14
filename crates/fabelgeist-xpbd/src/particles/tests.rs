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
            "capacity": usize::from(particles.capacity()),
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
        let requested = case["requested"].as_u64().unwrap() as u32;
        let mut particles = Particles::new(&context, requested.into()).unwrap();
        let states = case["states"].as_array().unwrap();
        assert_eq!(
            ParticleSnapshot::capture(&particles, &context).await.0,
            states[0]
        );
        let n = usize::from(particles.capacity());
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
        let capacity = particles
            .write(
                &context,
                &[Vec3::default(); 5],
                &[ParticleInverseMass::PINNED; 5],
            )
            .unwrap_err();
        let count = particles.write_positions(&context, &[]).unwrap_err();
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
