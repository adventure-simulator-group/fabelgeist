//! Select active authorized administrations for the medical notebook.

use super::AdministrationPresentation;
use adventuresim_world_schema::calendar::StrategicMinute;

pub(super) fn active(
    administrations: &[AdministrationPresentation],
    current_minute: StrategicMinute,
) -> Vec<AdministrationPresentation> {
    administrations
        .iter()
        .filter(|row| {
            row.stopped_at.is_none()
                && adventuresim_core::physiology::intervention_profile(
                    &row.preparation_id,
                    row.profile_version,
                )
                .is_some_and(|profile| {
                    current_minute >= row.administered_at
                        && current_minute
                            < row
                                .administered_at
                                .saturating_add_minutes(profile.duration_minutes)
                })
        })
        .cloned()
        .collect()
}
