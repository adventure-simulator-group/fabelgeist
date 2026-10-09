//! Stable deterministic primitives shared across dependency layers.
//!
//! Identical canonical inputs and draw sequences are reproducible across
//! supported targets. Intentional output changes require updating affected
//! replay markers and generated fixtures.

mod seed;
mod stream;
pub use seed::{Seed, StreamId};
pub use stream::{DeterministicRng, SamplingError};

const UNIT_F32_SCALE: f32 = (1_u32 << 24) as f32;
const INCLUSIVE_UNIT_F32_SCALE: f32 = ((1_u32 << 24) - 1) as f32;
const UNIT_F64_SCALE: f64 = (1_u64 << 53) as f64;
const INCLUSIVE_UNIT_F64_SCALE: f64 = ((1_u64 << 53) - 1) as f64;

/// Convert the high 24 bits to the half-open unit interval `[0, 1)`.
pub fn unit_f32(value: u64) -> f32 {
    (value >> 40) as f32 / UNIT_F32_SCALE
}

/// Convert the high 24 bits to the inclusive unit interval `[0, 1]`.
pub fn inclusive_unit_f32(value: u64) -> f32 {
    (value >> 40) as f32 / INCLUSIVE_UNIT_F32_SCALE
}

/// Convert the high 53 bits to the half-open unit interval `[0, 1)`.
pub fn unit_f64(value: u64) -> f64 {
    (value >> 11) as f64 / UNIT_F64_SCALE
}

/// Convert the high 53 bits to the inclusive unit interval `[0, 1]`.
pub fn inclusive_unit_f64(value: u64) -> f64 {
    (value >> 11) as f64 / INCLUSIVE_UNIT_F64_SCALE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_seed_zero_sequence_is_stable() {
        let mut random = DeterministicRng::new(Seed::from_u64(0));
        assert_eq!(random.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(random.next_u64(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(random.next_u64(), 0x06c4_5d18_8009_454f);
    }

    #[test]
    fn unit_conversions_keep_their_interval_contracts() {
        assert_eq!(unit_f32(0), 0.0);
        assert!(unit_f32(u64::MAX) < 1.0);
        assert_eq!(inclusive_unit_f32(u64::MAX), 1.0);
        assert_eq!(unit_f64(0), 0.0);
        assert!(unit_f64(u64::MAX) < 1.0);
        assert_eq!(inclusive_unit_f64(u64::MAX), 1.0);
    }
}
