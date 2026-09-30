//! Source notes for the reconstructed drought window.

use super::Sample;
use adventuresim_world_schema::{DroughtHistory, SummerHydroclimate, calendar::CalendarYear};

pub(super) fn source_note(year: CalendarYear, sample: Sample) -> String {
    let start = year
        .checked_sub_years(u16::from(DroughtHistory::WINDOW_YEARS) - 1)
        .expect("drought history starts within the calendar");
    let spatial = if sample.used_neighbor {
        "the containing cell was missing, so the nearest complete grid point within the 1.5° cutoff was used"
    } else {
        "the containing grid point was used"
    };
    history_note(year, start, sample.history, spatial, "reconstructed")
}

pub(super) fn fallback_note(year: CalendarYear, history: DroughtHistory) -> String {
    let start = year
        .checked_sub_years(u16::from(DroughtHistory::WINDOW_YEARS) - 1)
        .expect("drought history starts within the calendar");
    history_note(
        year,
        start,
        history,
        "no complete grid point was available within the 1.5° cutoff; a neutral fallback was used",
        "inferred",
    )
}

fn history_note(
    year: CalendarYear,
    start: CalendarYear,
    history: DroughtHistory,
    spatial: &str,
    classification: &str,
) -> String {
    format!(
        "**[NOAA OWDA v1.0](https://doi.org/10.25921/rjm6-mq74), [Cook et al.](https://doi.org/10.1126/sciadv.1500561):** Regional 0.5° summer PDSI estimate, not an exact settlement observation; {spatial}. {year}: {current} milli-PDSI ({condition}); {start}–{year} mean: {mean} milli-PDSI, rounded to the nearest milli-unit, with {dry} drought (≤ -2000) and {wet} wet (≥ 2000) summers. Profile is {classification}; tree-ring reconstruction and spatial assignment remain uncertain.",
        current = history.current_summer().milli_units(),
        mean = history.twenty_year_mean().milli_units(),
        dry = history.drought_summers(),
        wet = history.wet_summers(),
        condition = condition_name(history.current_summer().condition()),
    )
}

const fn condition_name(condition: SummerHydroclimate) -> &'static str {
    match condition {
        SummerHydroclimate::ExtremeDrought => "extreme drought",
        SummerHydroclimate::SevereDrought => "severe drought",
        SummerHydroclimate::ModerateDrought => "moderate drought",
        SummerHydroclimate::MildDrought => "mild drought",
        SummerHydroclimate::NearNormal => "near normal",
        SummerHydroclimate::MildlyWet => "mildly wet",
        SummerHydroclimate::ModeratelyWet => "moderately wet",
        SummerHydroclimate::VeryWet => "very wet",
        SummerHydroclimate::ExtremelyWet => "extremely wet",
    }
}
