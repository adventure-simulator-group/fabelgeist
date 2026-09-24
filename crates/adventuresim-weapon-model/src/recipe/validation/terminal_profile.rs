//! Positive-length fittings and explicit annular profile steps.
use super::*;
const MAX_TERMINAL_PROFILE_STATIONS: usize = 32;
pub(super) fn check(p: &GuardParameters) -> Checked {
    let selected = p.terminal == Some(GuardTerminal::Profile)
        || p.left_terminal == Some(GuardLeftTerminal::Profile)
        || p.right_terminal == Some(GuardRightTerminal::Profile);
    proportion(selected == p.terminal_profile.is_some())?;
    let Some(profile) = &p.terminal_profile else {
        return Ok(());
    };
    let stations = &profile.stations;
    require(
        (2..=MAX_TERMINAL_PROFILE_STATIONS).contains(&stations.len()),
        RecipeError::Budget,
    )?;
    require(
        stations[0][0].get() == 0.0 && stations.last().unwrap()[0].get() > 0.0,
        RecipeError::Profile,
    )?;
    for (i, station) in stations.iter().enumerate() {
        nonnegative(station[0].get())?;
        if i == stations.len() - 1 {
            nonnegative(station[1].get())?;
        } else {
            positive(station[1].get())?;
        }
    }
    for pair in stations.windows(2) {
        require(
            pair[1][0].get() >= pair[0][0].get() && pair[0] != pair[1],
            RecipeError::Profile,
        )?;
    }
    for triple in stations.windows(3) {
        require(
            triple[2][0].get() > triple[0][0].get(),
            RecipeError::Profile,
        )?;
    }
    Ok(())
}
