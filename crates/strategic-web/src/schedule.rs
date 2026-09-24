//! Schedule transport conversion; policy and sampling belong to core.
use adventuresim_core::{
    activity::ActivityLocation,
    strategic_schedule::{
        DailySchedule, ScheduleParseError, ValidatedSchedule, validate_daily_allocation,
    },
};
use adventuresim_stdb_client::ScheduleAllocation;

pub(crate) fn validate(
    mut schedule: ScheduleAllocation,
) -> Result<ScheduleAllocation, ScheduleParseError> {
    let normalize = |value: Option<String>| value.filter(|id| !id.trim().is_empty());
    let plan = validate_daily_allocation(
        [
            schedule.labor_minutes,
            schedule.prayer_minutes,
            schedule.thievery_minutes,
            schedule.raiding_minutes,
            schedule.combat_training_minutes,
            schedule.carousing_minutes,
            schedule.socializing_minutes,
            schedule.apprenticeship_minutes,
            schedule.profession_practice_minutes,
            schedule.reading_minutes,
        ],
        (
            schedule.apprenticeship_minutes,
            normalize(schedule.apprenticeship_organization_id),
        ),
        (
            schedule.profession_practice_minutes,
            normalize(schedule.practice_organization_id),
        ),
    )?;
    schedule.apprenticeship_organization_id = plan.apprenticeship.organization_id();
    schedule.practice_organization_id = plan.practice.organization_id();
    Ok(schedule)
}

pub(crate) fn effective(
    schedule: &ScheduleAllocation,
    location: ActivityLocation,
    character_id: u64,
) -> Result<ScheduleAllocation, ScheduleParseError> {
    let core = ValidatedSchedule::try_from(core_allocation(schedule))?
        .effective_at(location, character_id);
    Ok(ScheduleAllocation {
        reading_minutes: core.reading_minutes,
        combat_training_minutes: core.combat_training_minutes,
        carousing_minutes: core.carousing_minutes,
        socializing_minutes: core.socializing_minutes,
        apprenticeship_minutes: core.apprenticeship_minutes,
        profession_practice_minutes: core.profession_practice_minutes,
        labor_minutes: core.labor,
        prayer_minutes: core.prayer,
        thievery_minutes: core.thievery,
        raiding_minutes: core.raiding,
        ..schedule.clone()
    })
}

fn core_allocation(schedule: &ScheduleAllocation) -> DailySchedule {
    DailySchedule {
        reading_minutes: schedule.reading_minutes,
        combat_training_minutes: schedule.combat_training_minutes,
        carousing_minutes: schedule.carousing_minutes,
        socializing_minutes: schedule.socializing_minutes,
        apprenticeship_minutes: schedule.apprenticeship_minutes,
        profession_practice_minutes: schedule.profession_practice_minutes,
        labor: schedule.labor_minutes,
        prayer: schedule.prayer_minutes,
        thievery: schedule.thievery_minutes,
        raiding: schedule.raiding_minutes,
    }
}

#[derive(serde::Serialize)]
pub(crate) struct SchedulePreview {
    pub effective: DailySchedule,
    pub leisure_minutes: u64,
}

impl SchedulePreview {
    pub fn calculate(
        schedule: &ScheduleAllocation,
        location: ActivityLocation,
        character_id: u64,
    ) -> Result<Self, ScheduleParseError> {
        let effective = ValidatedSchedule::try_from(core_allocation(schedule))?
            .effective_at(location, character_id);
        Ok(Self {
            effective,
            leisure_minutes: adventuresim_core::strategic_time::MINUTES_PER_DAY
                - effective.allocated_minutes(),
        })
    }
}
