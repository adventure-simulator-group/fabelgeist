use super::*;

#[test]
#[cfg(target_pointer_width = "64")]
fn rejects_host_overflow_without_losing_the_full_input_count() {
    let input = ParticleInputCount::from(u32::MAX as usize + 1);
    assert_eq!(
        ParticleCount::try_from(input),
        Err(ParticleCountOverflow { input })
    );
    assert_eq!(
        input
            .admit_masses(InverseMassCount::from(0))
            .unwrap_err()
            .to_string(),
        "Particles::write: 4294967296 positions but 0 inverse masses"
    );
    input
        .admit_masses(InverseMassCount::from(usize::from(input)))
        .unwrap();
}
