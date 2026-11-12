//! Deterministic randomness.
//!
//! Blender's hair nodes draw their randomness from `Random Value` nodes keyed
//! by an ID (a curve's stable ID, or a point's index) and a seed, so that a
//! curve's curl phase or tip spread does not change when other curves are
//! added or removed. [`unit`] and friends are that: stateless hashes of an ID
//! and a seed. [`Rng`] is a sequential source for the few places that draw in
//! order -- placing roots, choosing a dataset sample's parameters.

use fabelgeist_math::Vec3;

/// SplitMix64: tiny, fast, and statistically fine for placing roots.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in `[low, high)`.
    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next_f32()
    }

    /// Uniform in `[low, high)`, but spread evenly on a log scale: for
    /// parameters like a curl's frequency, where 5 to 10 is as big a step as
    /// 20 to 40.
    pub fn log_range(&mut self, low: f32, high: f32) -> f32 {
        (low.ln() + (high.ln() - low.ln()) * self.next_f32()).exp()
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }

    /// An index into `0..n`.
    pub fn index(&mut self, n: usize) -> usize {
        ((self.next_f32() * n as f32) as usize).min(n.saturating_sub(1))
    }

    /// A standard normal draw (Box-Muller).
    pub fn gaussian(&mut self) -> f32 {
        let u = self.next_f32().max(1e-7);
        let v = self.next_f32();
        (-2.0 * u.ln()).sqrt() * (std::f32::consts::TAU * v).cos()
    }

    /// A fresh generator derived from this one.
    pub fn fork(&mut self) -> Self {
        Self::new(self.next_u64())
    }
}

/// Draws in order from one word: each the hash of the key and a counter.
/// Where [`Rng`] keeps sixty-four bits of state, this keeps a counter, so a
/// GPU thread draws the very same numbers ([`crate::gpu`]): the guides are
/// grown with it.
#[derive(Clone, Debug)]
pub struct Stream {
    key: u32,
    counter: u32,
}

impl Stream {
    pub fn new(key: u32) -> Self {
        Self { key, counter: 0 }
    }

    pub fn next_u32(&mut self) -> u32 {
        let value = hash(self.key, self.counter);
        self.counter = self.counter.wrapping_add(1);
        value
    }

    /// Uniform in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// A standard normal draw (Box-Muller).
    pub fn gaussian(&mut self) -> f32 {
        let u = self.next_f32().max(1e-7);
        let v = self.next_f32();
        (-2.0 * u.ln()).sqrt() * (std::f32::consts::TAU * v).cos()
    }
}

/// A 32-bit avalanche of two words (the finaliser of MurmurHash3 over a
/// combined key).
pub fn hash(a: u32, b: u32) -> u32 {
    let mut h = a.wrapping_mul(0x9E37_79B1) ^ b.rotate_left(16).wrapping_mul(0x85EB_CA6B);
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^ (h >> 16)
}

/// Three words into one.
pub fn hash3(a: u32, b: u32, c: u32) -> u32 {
    hash(hash(a, b), c)
}

/// Uniform in `[0, 1)` for an ID under a seed.
pub fn unit(id: u32, seed: u32) -> f32 {
    (hash(id, seed) >> 8) as f32 / (1u32 << 24) as f32
}

/// Uniform in `[-1, 1)` for an ID under a seed.
pub fn signed(id: u32, seed: u32) -> f32 {
    unit(id, seed) * 2.0 - 1.0
}

/// Each component uniform in `[-1, 1)`: Blender's vector `Random Value`
/// between -1 and 1.
pub fn signed_vec3(id: u32, seed: u32) -> Vec3 {
    Vec3::new(
        signed(id, hash(seed, 1)),
        signed(id, hash(seed, 2)),
        signed(id, hash(seed, 3)),
    )
}

/// A seed from a user-facing integer, as Blender's `Hash Value` node does
/// before feeding a seed to `Random Value`.
pub fn seed(value: i32, salt: u32) -> u32 {
    hash(value as u32, salt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_are_uniform_enough() {
        let n = 20_000;
        let mean: f32 = (0..n).map(|i| unit(i, 7)).sum::<f32>() / n as f32;
        assert!((mean - 0.5).abs() < 0.01, "{mean}");
        assert!((0..n).all(|i| (0.0..1.0).contains(&unit(i, 3))));
    }

    #[test]
    fn seeds_decorrelate() {
        let same = (0..1000)
            .filter(|&i| (unit(i, 1) - unit(i, 2)).abs() < 1e-3)
            .count();
        assert!(same < 10);
    }
}
