//! Deterministic daily location choices with a stable identity tie break.

use super::stable_lifecycle_hash;
use adventuresim_world_schema::calendar::StrategicDayIndex;

pub fn daily_location_target_score(
    actor_id: &str,
    location_id: &str,
    calendar_day: StrategicDayIndex,
    target_id: &str,
) -> u64 {
    let day = calendar_day.to_string();
    stable_lifecycle_hash(
        "daily-location-target",
        &[actor_id, location_id, &day, target_id],
    )
}

pub fn select_stable_target_by_score<'a>(
    candidates: impl IntoIterator<Item = &'a str>,
    score: impl Fn(&str) -> u64,
) -> Option<&'a str> {
    candidates
        .into_iter()
        .min_by_key(|candidate| (score(candidate), *candidate))
}

/// Pick the lowest deterministic score. Character ID is the stable final tie
/// break, so storage or iteration order cannot affect an ambiguous choice.
pub fn select_daily_location_target<'a>(
    actor_id: &str,
    location_id: &str,
    calendar_day: StrategicDayIndex,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<&'a str> {
    select_stable_target_by_score(candidates, |candidate| {
        daily_location_target_score(actor_id, location_id, calendar_day, candidate)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_target_selection_is_order_independent_and_ties_by_id() {
        let forward = select_daily_location_target(
            "actor",
            "wittenberg",
            StrategicDayIndex::new(42),
            ["c", "a", "b"],
        )
        .unwrap();
        let reverse = select_daily_location_target(
            "actor",
            "wittenberg",
            StrategicDayIndex::new(42),
            ["b", "a", "c"],
        )
        .unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(
            select_stable_target_by_score(["zeta", "alpha"], |_| 7),
            Some("alpha")
        );
        assert_ne!(
            daily_location_target_score("actor", "wittenberg", StrategicDayIndex::new(42), "a"),
            daily_location_target_score("actor", "wittenberg", StrategicDayIndex::new(43), "a")
        );
    }
}
