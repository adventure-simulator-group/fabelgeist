//! The independent original scalar solver pins words, not just a tolerance.
use super::*;
use crate::native_vector_fixture::{decode_native_vectors, encode_native_vectors};

#[test]
fn preserves_original_mass_weighted_contact_words() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/contact_masses.json"))
            .expect("original contact snapshot");
    let cases = fixture["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 36);
    for (pair, family) in [Pair::VertexTriangle, Pair::EdgeEdge]
        .into_iter()
        .zip(cases.as_chunks::<18>().0)
    {
        for case in family {
            assert_eq!(case["pair"], format!("{pair:?}"));
            let previous = decode_native_vectors(&case["previous"]);
            let mut positions = decode_native_vectors(&case["predicted"]);
            let mut velocities = decode_native_vectors(&case["initial_velocities"]);
            let mut masses = Vec::new();
            for word in case["inverse_masses"]
                .as_array()
                .expect("inverse mass words")
            {
                masses.push(ParticleInverseMass::from(f32::from_bits(
                    word.as_u64().expect("inverse mass word") as u32,
                )));
            }
            let with_velocity = case["with_velocity"].as_bool().expect("velocity choice");
            let count = resolve(
                pair,
                &mut positions,
                &previous,
                &masses,
                [0, 1, 2, 3],
                f32::from_bits(case["thickness"].as_u64().expect("thickness word") as u32),
                with_velocity.then_some(velocities.as_mut_slice()),
            );
            assert_eq!(count.to_string(), case["resolved"].to_string());
            assert_eq!(encode_native_vectors(&positions), case["corrected"]);
            assert_eq!(encode_native_vectors(&velocities), case["velocities"]);
        }
    }
}
