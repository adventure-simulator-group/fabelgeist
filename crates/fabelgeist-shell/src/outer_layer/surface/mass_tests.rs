//! Original outer-layer snapshots include overflow and nonfinite mass response.
use super::*;
use crate::native_vector_fixture::{decode_native_vectors, encode_native_vectors};

#[test]
fn preserves_original_layer_projection_and_residual_words() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/layer_masses.json"))
            .expect("original layer snapshot");
    let cases = fixture["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 24);
    for case in cases {
        let layer = OuterLayer::new(
            vec![
                Vec3::new(-0.1, -0.1, 0.0),
                Vec3::new(0.0, 0.1, 0.0),
                Vec3::new(0.1, -0.1, 0.0),
            ],
            vec![[0, 1, 2]],
            0.003,
            Vec3::new(0.0, 0.0, -1.0),
        );
        let mut positions = decode_native_vectors(&case["positions"]);
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
        let changed = layer.project(&mut positions, &mut velocities, &masses, &[[0, 1, 2]]);
        assert_eq!(serde_json::json!(changed), case["changed"]);
        assert_eq!(encode_native_vectors(&positions), case["corrected"]);
        assert_eq!(encode_native_vectors(&velocities), case["velocities"]);
        assert_eq!(
            serde_json::json!(layer.surface_residual(&positions, &[[0, 1, 2]]).to_bits()),
            case["residual"]
        );
    }
}
