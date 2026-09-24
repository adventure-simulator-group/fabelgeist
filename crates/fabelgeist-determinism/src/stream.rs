//! Sampling policy: all randomness flows through fixed-width `next_u64`.

use std::num::NonZeroU64;

use rand::{
    RngCore, SeedableRng,
    distr::{Distribution, Uniform, weighted::WeightedIndex},
};

use crate::Seed;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SamplingError {
    EmptyCandidates,
    ZeroTotalWeight,
    WeightOverflow,
}

impl std::fmt::Display for SamplingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyCandidates => "sampling requires candidates",
            Self::ZeroTotalWeight => "sampling requires positive total weight",
            Self::WeightOverflow => "sampling weights exceed u64",
        })
    }
}
impl std::error::Error for SamplingError {}

/// Portable SplitMix64 draws and integer sampling, with no ambient entropy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicRng(rand_xoshiro::SplitMix64);

impl DeterministicRng {
    pub fn new(seed: Seed) -> Self {
        Self(rand_xoshiro::SplitMix64::from_seed(seed.to_le_bytes()))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// Uniform `[0, upper)` using Rand's constructed, unbiased u64 sampler.
    pub fn below(&mut self, upper: NonZeroU64) -> u64 {
        sample_below(&mut self.0, upper)
    }

    /// Select an index only after sampling at a fixed width.
    ///
    /// Panics for an empty collection in every build profile. Callers with
    /// optional candidates must handle absence before requesting a draw.
    pub fn index(&mut self, length: usize) -> usize {
        let bound =
            NonZeroU64::new(length as u64).expect("sampling requires a nonempty collection");
        usize::try_from(self.below(bound)).expect("sample fits collection length")
    }

    pub fn boolean(&mut self) -> bool {
        self.below(NonZeroU64::new(2).expect("two outcomes")) == 0
    }

    pub fn unit_f32(&mut self) -> f32 {
        crate::unit_f32(self.next_u64())
    }

    pub fn unit_f64(&mut self) -> f64 {
        crate::unit_f64(self.next_u64())
    }

    pub fn inclusive_unit_f32(&mut self) -> f32 {
        crate::inclusive_unit_f32(self.next_u64())
    }

    pub fn inclusive_unit_f64(&mut self) -> f64 {
        crate::inclusive_unit_f64(self.next_u64())
    }

    /// Scaling has interval semantics but no downstream bitwise guarantee.
    pub fn range_f32(&mut self, minimum: f32, maximum: f32) -> f32 {
        assert!(minimum.is_finite() && maximum.is_finite() && minimum <= maximum);
        minimum + self.unit_f32() * (maximum - minimum)
    }

    /// Descending Fisher–Yates. Every swap uses the same u64 sampler.
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let selected = self.index(index + 1);
            values.swap(index, selected);
        }
    }

    /// The input order must be authored or canonicalized by stable identity.
    pub fn weighted_index(&mut self, weights: &[u64]) -> Result<usize, SamplingError> {
        use rand::distr::weighted::Error;
        let distribution = WeightedIndex::<u64>::new(weights).map_err(|error| match error {
            Error::InvalidInput => SamplingError::EmptyCandidates,
            Error::InsufficientNonZero => SamplingError::ZeroTotalWeight,
            Error::Overflow => SamplingError::WeightOverflow,
            Error::InvalidWeight => unreachable!("u64 weights are nonnegative"),
            _ => unreachable!("pinned weighted sampler error"),
        })?;
        Ok(distribution.sample(&mut self.0))
    }

    /// Proportional draws without replacement; zero-weight entries are omitted.
    pub fn weighted_order(&mut self, weights: &[u64]) -> Result<Vec<usize>, SamplingError> {
        let mut remaining = weights.to_vec();
        let mut result = Vec::new();
        let count = remaining.iter().filter(|weight| **weight != 0).count();
        if count == 0 {
            return self.weighted_index(weights).map(|_| result);
        }
        for _ in 0..count {
            let index = self.weighted_index(&remaining)?;
            result.push(index);
            remaining[index] = 0;
        }
        Ok(result)
    }
}

fn sample_below(random: &mut impl RngCore, upper: NonZeroU64) -> u64 {
    Uniform::new(0_u64, upper.get())
        .expect("nonzero bound")
        .sample(random)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RejectionProbe(usize);
    impl RngCore for RejectionProbe {
        fn next_u32(&mut self) -> u32 {
            panic!("sampling must use u64")
        }
        fn next_u64(&mut self) -> u64 {
            let result = [0, 1][self.0];
            self.0 += 1;
            result
        }
        fn fill_bytes(&mut self, _: &mut [u8]) {
            panic!("sampling must use u64")
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn rejection_consumes_another_full_width_draw() {
        let mut random = RejectionProbe(0);
        assert_eq!(
            sample_below(&mut random, NonZeroU64::new(u64::MAX).unwrap()),
            0
        );
        assert_eq!(random.0, 2);
    }
}
