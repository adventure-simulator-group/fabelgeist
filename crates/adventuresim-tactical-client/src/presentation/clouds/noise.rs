//! Cloud lattice samples and weather-context seed derivation.
use super::*;
use fabelgeist_determinism::Seed;

pub(super) fn non_periodic_value_noise_3d(position: CloudDensityCoordinate, seed: Seed) -> f32 {
    // Native numerical kernel: dimensionless density-lattice axes, never metres.
    let position = position.axes();
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

pub(super) fn cloud_seed(environment: &SceneEnvironment) -> Seed {
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
}

/// Dimensionless displacement of the density lattice; never a world position.
pub(super) fn density_warp(
    coordinate: CloudDensityCoordinate,
    seed: fabelgeist_determinism::Seed,
    slot: ActiveCloudLayerOrdinal,
) -> CloudDensityWarp {
    use super::streams;
    CloudDensityWarp(
        Vec3::new(
            non_periodic_value_noise_3d(
                coordinate * 0.36,
                streams::WARP_X.seed(seed, &[slot.context_word()]),
            ),
            non_periodic_value_noise_3d(
                coordinate * 0.36 + Vec3::splat(13.7),
                streams::WARP_Y.seed(seed, &[slot.context_word()]),
            ),
            non_periodic_value_noise_3d(
                coordinate * 0.36 + Vec3::new(4.1, 9.7, 17.3),
                streams::WARP_Z.seed(seed, &[slot.context_word()]),
            ),
        ) - Vec3::splat(0.5),
    )
}

/// Coordinate in the dimensionless three-axis cloud density lattice.
/// World metres are scaled by the layer recipe before this value is produced.
#[derive(Clone, Copy)]
pub(super) struct CloudDensityCoordinate(Vec3);

impl CloudDensityCoordinate {
    /// Native axes at the owning layer-to-lattice numerical boundary.
    pub(super) fn from_layer_axes(axes: Vec3) -> Self {
        Self(axes)
    }
    pub(super) fn axes(self) -> Vec3 {
        self.0
    }
}

impl std::ops::Mul<f32> for CloudDensityCoordinate {
    type Output = Self;
    fn mul(self, frequency: f32) -> Self {
        Self(self.0 * frequency)
    }
}
impl std::ops::Add<Vec3> for CloudDensityCoordinate {
    type Output = Self;
    /// Native authored offsets are dimensionless lattice axes.
    fn add(self, offset: Vec3) -> Self {
        Self(self.0 + offset)
    }
}
impl std::ops::Add<CloudDensityWarp> for CloudDensityCoordinate {
    type Output = Self;
    fn add(self, warp: CloudDensityWarp) -> Self {
        Self(self.0 + warp.0)
    }
}

/// Dimensionless centred density noise, scaled before displacing a coordinate.
#[derive(Clone, Copy)]
pub(super) struct CloudDensityWarp(Vec3);
impl std::ops::Mul<Vec3> for CloudDensityWarp {
    type Output = Self;
    /// Native axis gains belong to the density numerical kernel.
    fn mul(self, gains: Vec3) -> Self {
        Self(self.0 * gains)
    }
}

/// Ordinal among active layers after absent bake layers have been filtered out.
/// This preserves the existing ordered stream context; it is not a profile ID.
#[derive(Clone, Copy)]
pub(super) struct ActiveCloudLayerOrdinal(usize);
impl ActiveCloudLayerOrdinal {
    pub(super) const fn new(index: usize) -> Self {
        Self(index)
    }
    pub(super) const fn context_word(self) -> u64 {
        self.0 as u64
    }
}
