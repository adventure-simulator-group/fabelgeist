//! Frozen observer/day coordinates for deterministic secret-courtship trials.

use crate::identity::CharacterId;
use adventuresim_world_schema::calendar::StrategicDayIndex;
use fabelgeist_determinism::{DeterministicRng, Seed, StreamId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryDayOutcome {
    Evaluated,
    DeferredForFrontier,
}

/// One observer's trial for the unordered canonical pair on an absolute day.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CourtshipDiscoveryTrial {
    first: CharacterId,
    second: CharacterId,
    observer: CharacterId,
    day: StrategicDayIndex,
}

impl CourtshipDiscoveryTrial {
    pub fn new(
        first: CharacterId,
        second: CharacterId,
        observer: CharacterId,
        day: StrategicDayIndex,
    ) -> Self {
        Self {
            first: first.min(second),
            second: first.max(second),
            observer,
            day,
        }
    }

    /// Preserve the numeric stream's fixed-width little-endian framing.
    pub fn rng(self) -> DeterministicRng {
        let first = u64::from(self.first).to_le_bytes();
        let second = u64::from(self.second).to_le_bytes();
        let observer = u64::from(self.observer).to_le_bytes();
        let day = u64::from(self.day).to_le_bytes();
        Seed::derive(
            &first,
            StreamId::new("courtship.discovery"),
            &[&second, &observer, &day],
        )
        .rng()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trial_preserves_historical_numeric_frames_and_canonical_pair_order() {
        for (first, second, observer, day, expected) in [
            (7, 17, 19, 23, [1058736282, 1057667890]),
            (7, 17, 19, 24, [1061065204, 1060681499]),
            (0, u64::MAX, 1, u64::MAX, [1064228113, 1058409251]),
        ] {
            let trial = CourtshipDiscoveryTrial::new(
                first.into(),
                second.into(),
                observer.into(),
                day.into(),
            );
            let mut random = trial.rng();
            assert_eq!(
                [random.unit_f32().to_bits(), random.unit_f32().to_bits()],
                expected
            );
            assert_eq!(
                trial,
                CourtshipDiscoveryTrial::new(
                    second.into(),
                    first.into(),
                    observer.into(),
                    day.into()
                )
            );
        }
    }
}
