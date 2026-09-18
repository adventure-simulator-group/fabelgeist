//! Purpose-separated foraging draws keyed by source and item identity.
use fabelgeist_determinism::{Seed, StreamId};

pub(super) enum ForageDraw {
    FractionalYield,
    BaseYield,
    Stealth,
}

impl ForageDraw {
    pub(super) fn below(self, seed: u64, source: &str, item: &str, upper: u64) -> u64 {
        let purpose = StreamId::new(match self {
            Self::FractionalYield => "foraging.fractional-yield",
            Self::BaseYield => "foraging.base-yield",
            Self::Stealth => "foraging.stealth",
        });
        Seed::derive(
            &seed.to_le_bytes(),
            purpose,
            &[source.as_bytes(), item.as_bytes()],
        )
        .rng()
        .below(std::num::NonZeroU64::new(upper).expect("forage yield bounds are positive"))
    }
}
