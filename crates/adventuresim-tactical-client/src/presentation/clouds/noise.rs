//! Cloud lattice samples and weather-context seed derivation.
use super::*;

pub(super) fn non_periodic_value_noise_3d(position: Vec3, seed: u64) -> f32 {
    let cell = position.floor();
    let fraction = position - cell;
    // Quintic interpolation makes both first and second derivatives vanish at
    // lattice boundaries. The cloud field is magnified over kilometres, so
    // the cubic value-noise shoulder was still legible as broad square cells.
    let smooth = fraction
        * fraction
        * fraction
        * (fraction * (fraction * 6.0 - Vec3::splat(15.0)) + Vec3::splat(10.0));
    let value = |offset: Vec3| {
        let lattice = cell + offset;
        streams::LATTICE
            .rng(
                seed,
                &[
                    lattice.x as i64 as u64,
                    lattice.y as i64 as u64,
                    lattice.z as i64 as u64,
                ],
            )
            .inclusive_unit_f32()
    };
    let x0 = value(Vec3::ZERO).lerp(value(Vec3::X), smooth.x);
    // The z=0 upper-X corner is (1, 1, 0). Sampling (1, 1, 1) here coupled
    // adjacent Z cells and exposed axis-aligned macro blocks in the dome bake.
    let x1 = value(Vec3::Y).lerp(value(Vec3::X + Vec3::Y), smooth.x);
    let y0 = x0.lerp(x1, smooth.y);
    let x2 = value(Vec3::Z).lerp(value(Vec3::Z + Vec3::X), smooth.x);
    let x3 = value(Vec3::Z + Vec3::Y).lerp(value(Vec3::ONE), smooth.x);
    y0.lerp(x2.lerp(x3, smooth.y), smooth.z)
}

pub(super) fn cloud_seed(environment: &SceneEnvironment) -> u64 {
    fabelgeist_determinism::Seed::derive(
        environment.scene_digest.as_bytes(),
        streams::ENVIRONMENT,
        &[
            &(environment.absolute_minute / 360).to_le_bytes(),
            &environment.latitude_microdegrees.to_le_bytes(),
            &environment.longitude_microdegrees.to_le_bytes(),
        ],
    )
    .to_u64()
}
