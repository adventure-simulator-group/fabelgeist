//! Stable draw ownership for battle loot.

use std::num::NonZeroU64;

use fabelgeist_determinism::StreamId;

const GOLD_PRESENCE: StreamId = StreamId::new("loot.gold-presence");
const GOLD_AMOUNT: StreamId = StreamId::new("loot.gold-amount");

pub(super) fn gold(seed: u64, maximum: u32) -> Option<u32> {
    GOLD_PRESENCE.rng(seed, &[]).boolean().then(|| {
        1 + GOLD_AMOUNT
            .rng(seed, &[])
            .below(NonZeroU64::new(u64::from(maximum)).expect("positive loot maximum"))
            as u32
    })
}
