//! Native admission and the massless policy keep their distinct contracts.
use super::*;
use crate::ParticleIndex;

#[test]
fn inverse_native_words_are_preserved_before_cpu_admission() {
    for word in [0x80000000u32, 0x7fc01234, 0x7f800000, 0xbf800000] {
        let mass = ParticleInverseMass::from(f32::from_bits(word));
        assert_eq!(u32::from(mass), word);
    }
    assert_eq!(
        ParticleInverseMass::from(-0.0).mobility(),
        ParticleMobility::Prescribed
    );
    assert_eq!(
        ParticleInverseMass::from(-0.0).validity(),
        MassValidity::FiniteNonnegative
    );
    assert_eq!(
        ParticleInverseMass::from(f32::NAN).validity(),
        MassValidity::Invalid
    );
    assert_eq!(
        ParticleInverseMass::from(-1.0).validity(),
        MassValidity::Invalid
    );
    assert_eq!(
        ParticleInverseMass::from(f32::INFINITY).validity(),
        MassValidity::Invalid
    );
}

#[test]
fn massless_threshold_retains_strict_admission_and_signed_zero_policy() {
    let threshold = ParticleMass::MASSLESS_THRESHOLD_KG;
    for value in [-0.0, 0.0, threshold, -1.0, f32::NAN] {
        assert_eq!(u32::from(ParticleMass::from(value).inverse_mass()), 0);
    }
    let above = f32::from_bits(threshold.to_bits() + 1);
    assert_eq!(
        u32::from(ParticleMass::from(above).inverse_mass()),
        (1.0 / above).to_bits()
    );
    assert_eq!(
        ParticleMass::from(4.0).inverse_mass(),
        ParticleInverseMass::from(0.25)
    );
    assert_eq!(
        u32::from(ParticleMass::from(f32::INFINITY).inverse_mass()),
        0
    );
}

#[test]
fn areal_density_admission_is_separate_from_native_construction() {
    for value in [0.0, -0.0, -1.0, f32::INFINITY, f32::NAN] {
        assert_eq!(
            ParticleArealDensity::from(value).validity(),
            ArealDensityValidity::Invalid
        );
    }
    assert_eq!(
        ParticleArealDensity::from(f32::MIN_POSITIVE).validity(),
        ArealDensityValidity::PositiveFinite
    );
}

#[test]
fn preserves_original_triangle_mass_inverse_policy_and_native_sum_words() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/vertex_masses.json"))
            .expect("original triangle-mass snapshot");
    let cases = fixture["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 40);
    for case in cases {
        let mut points = Vec::new();
        for words in case["positions"].as_array().expect("position words") {
            points.push(Vec3::new(
                f32::from_bits(words[0].as_u64().expect("x word") as u32),
                f32::from_bits(words[1].as_u64().expect("y word") as u32),
                f32::from_bits(words[2].as_u64().expect("z word") as u32),
            ));
        }
        let density = ParticleArealDensity::from(f32::from_bits(
            case["density"].as_u64().expect("density word") as u32,
        ));
        let mut masses = vec![ParticleMass::ZERO; points.len()];
        for triangle in [
            [0u32.into(), 1u32.into(), 2u32.into()],
            [1u32.into(), 3u32.into(), 2u32.into()],
        ] {
            let share = density
                .vertex_share(triangle.map(|i: ParticleIndex| -> Vec3 { points[usize::from(i)] }));
            for vertex in triangle {
                masses[usize::from(vertex)] += share;
            }
        }
        let mut mass_words = Vec::new();
        let mut inverse_words = Vec::new();
        for mass in &masses {
            mass_words.push(mass.0.to_bits());
            inverse_words.push(u32::from(mass.inverse_mass()));
        }
        let total: ParticleMass = masses.iter().sum();
        assert_eq!(serde_json::json!(mass_words), case["mass_words"]);
        assert_eq!(serde_json::json!(inverse_words), case["inverse_words"]);
        assert_eq!(serde_json::json!(total.0.to_bits()), case["total_word"]);
    }
}
