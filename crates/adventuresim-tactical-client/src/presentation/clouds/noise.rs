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
                seed.into(),
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
            &environment
                .absolute_minute
                .period_index(360)
                .expect("cloud seed period must be nonzero")
                .to_le_bytes(),
            &environment.latitude_microdegrees.get().to_le_bytes(),
            &environment.longitude_microdegrees.get().to_le_bytes(),
        ],
    )
    .to_u64()
}

/// Native finite 3-D density lattice domain warp; seed and framing stay canonical.
pub(super) fn density_warp(
    coordinate: Vec3,
    seed: fabelgeist_determinism::Seed,
    slot: u64,
) -> Vec3 {
    use super::streams;
    Vec3::new(
        non_periodic_value_noise_3d(
            coordinate * 0.36,
            streams::WARP_X.seed(seed, &[slot]).to_u64(),
        ),
        non_periodic_value_noise_3d(
            coordinate * 0.36 + Vec3::splat(13.7),
            streams::WARP_Y.seed(seed, &[slot]).to_u64(),
        ),
        non_periodic_value_noise_3d(
            coordinate * 0.36 + Vec3::new(4.1, 9.7, 17.3),
            streams::WARP_Z.seed(seed, &[slot]).to_u64(),
        ),
    ) - Vec3::splat(0.5)
}
