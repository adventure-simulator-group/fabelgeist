//! A small deterministic random source.
//!
//! Every plant carries its own, seeded from the ecosystem's, so that a plant's
//! development does not depend on the order plants are stepped in -- which is
//! what lets them step in parallel and still reproduce.

use fabelgeist_determinism::Seed;

/// SplitMix64: tiny, fast, and statistically fine for placing seeds.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: Seed) -> Self {
        Self(seed.to_u64() ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in `[low, high)`.
    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next_f32()
    }

    /// A standard normal draw (Box-Muller).
    pub fn gaussian(&mut self) -> f32 {
        let u = self.next_f32().max(1e-7);
        let v = self.next_f32();
        (-2.0 * u.ln()).sqrt() * (std::f32::consts::TAU * v).cos()
    }

    /// Round `x` up or down at random so that the expectation is `x`.
    pub fn stochastic_round(&mut self, x: f32) -> u32 {
        let floor = x.max(0.0).floor();
        floor as u32 + u32::from(self.next_f32() < x - floor)
    }

    /// A fresh generator derived from this one.
    pub fn fork(&mut self) -> Self {
        Self::new(Seed::from_u64(self.next_u64()))
    }
}
