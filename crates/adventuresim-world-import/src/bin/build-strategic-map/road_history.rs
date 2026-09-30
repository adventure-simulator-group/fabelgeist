//! Evaluate dated road features against the validated world year.

use std::collections::BTreeMap;

use adventuresim_world_schema::calendar::CalendarYear;

pub(super) fn active(row: &BTreeMap<String, String>, year: CalendarYear) -> bool {
    let from = row
        .get("fromyear")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(i32::MIN);
    let to = row
        .get("toyear")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(i32::MAX);
    from <= year.get() && year.get() < to
}
