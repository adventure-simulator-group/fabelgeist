use super::*;

#[test]
#[cfg(target_pointer_width = "64")]
fn preserves_full_host_admission_before_native_narrowing() {
    let input = ParticleInputCount::from(u32::MAX as usize + 1);
    assert_eq!(input.gpu_count(), ParticleCount::EMPTY);
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
    ParticleCapacity::from(0).admit(input).unwrap();
    ParticleCount::EMPTY.admit_replacement(input).unwrap();
}
