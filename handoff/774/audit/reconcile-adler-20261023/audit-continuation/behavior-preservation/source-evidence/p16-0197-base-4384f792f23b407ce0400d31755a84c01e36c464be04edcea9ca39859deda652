//! Project persisted interval inputs for disease decisions.

use super::*;

pub(super) fn disease_events(
    episodes: &[InfectionEpisode],
    from: StrategicMinute,
    to: StrategicMinute,
    immunity: f32,
) -> Vec<disease::DiseaseEvent> {
    episodes
        .iter()
        .copied()
        .flat_map(|episode| disease::interval_events(episode, from, to, immunity))
        .collect()
}

pub(super) fn contact_windows(
    pairs: Vec<CachedPairPresence>,
    starts: &BTreeMap<u64, StrategicMinute>,
    horizons: &BTreeMap<u64, StrategicMinute>,
) -> Vec<disease::ContactWindow> {
    pairs
        .into_iter()
        .filter_map(|pair| {
            let participant_starts = [pair.low_id, pair.high_id]
                .into_iter()
                .filter_map(|id| starts.get(&id).copied())
                .collect::<Vec<_>>();
            let participant_horizons = [pair.low_id, pair.high_id]
                .into_iter()
                .filter_map(|id| horizons.get(&id).copied())
                .collect::<Vec<_>>();
            let start = participant_starts
                .into_iter()
                .max()
                .unwrap_or(pair.start)
                .saturating_add_minutes(1)
                .max(pair.start);
            let end = participant_horizons
                .into_iter()
                .min()
                .unwrap_or(pair.end)
                .min(pair.end);
            (start <= end).then_some(disease::ContactWindow {
                low_id: pair.low_id,
                high_id: pair.high_id,
                start,
                end,
            })
        })
        .collect()
}
