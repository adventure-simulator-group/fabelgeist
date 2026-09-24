//! Validated daily allocations and read-only, location-dependent redistribution.
use crate::activity::{ACTIVITY_SEGMENT_MINUTES, ActivityLocation, LocationActivity};
use crate::strategic_time::MINUTES_PER_DAY;
use fabelgeist_determinism::StreamId;

/// A parsed organization allocation: zero minutes cannot carry a stale
/// organization and positive minutes cannot omit the organization that grants
/// the activity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrganizationAllocation {
    None,
    Scheduled {
        minutes: ActivityMinutes,
        organization_id: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActivityMinutes(u16);

impl ActivityMinutes {
    pub fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for ActivityMinutes {
    type Error = ScheduleParseError;

    fn try_from(minutes: u16) -> Result<Self, Self::Error> {
        minutes
            .is_multiple_of(ACTIVITY_SEGMENT_MINUTES)
            .then_some(Self(minutes))
            .ok_or(ScheduleParseError::NotQuarterHour)
    }
}

impl OrganizationAllocation {
    pub fn parse(
        minutes: u16,
        organization_id: Option<String>,
    ) -> Result<Self, ScheduleParseError> {
        match (minutes, organization_id) {
            (0, None) => Ok(Self::None),
            (0, Some(_)) => Err(ScheduleParseError::OrganizationWithoutMinutes),
            (_, Some(organization_id)) => {
                let organization_id = organization_id.trim();
                if organization_id.is_empty() {
                    return Err(ScheduleParseError::EmptyOrganization);
                }
                Ok(Self::Scheduled {
                    minutes: ActivityMinutes::try_from(minutes)?,
                    organization_id: organization_id.to_owned(),
                })
            }
            (_, None) => Err(ScheduleParseError::MinutesWithoutOrganization),
        }
    }

    pub fn minutes(&self) -> u16 {
        match self {
            Self::None => 0,
            Self::Scheduled { minutes, .. } => minutes.get(),
        }
    }

    pub fn organization_id(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Scheduled {
                organization_id, ..
            } => Some(organization_id.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleParseError {
    ExceedsDay,
    NotQuarterHour,
    OrganizationWithoutMinutes,
    MinutesWithoutOrganization,
    EmptyOrganization,
}

impl std::fmt::Display for ScheduleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ExceedsDay => "The downtime plan must fit within 24 hours",
            Self::NotQuarterHour => "Schedule allocations must use 15-minute increments",
            Self::OrganizationWithoutMinutes => {
                "An organization cannot be selected without scheduled minutes"
            }
            Self::MinutesWithoutOrganization => {
                "Scheduled organization activity requires an organization"
            }
            Self::EmptyOrganization => "Scheduled organization ID cannot be blank",
        })
    }
}
impl std::error::Error for ScheduleParseError {}

/// Validates the cross-field invariants of a daily allocation at the reducer
/// boundary. Organization policy/membership remains an authoritative DB check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailySchedulePlan {
    pub activities: [ActivityMinutes; 10],
    pub apprenticeship: OrganizationAllocation,
    pub practice: OrganizationAllocation,
}

pub fn validate_daily_allocation(
    minutes: [u16; 10],
    apprenticeship: (u16, Option<String>),
    practice: (u16, Option<String>),
) -> Result<DailySchedulePlan, ScheduleParseError> {
    if minutes.into_iter().map(u64::from).sum::<u64>() > MINUTES_PER_DAY {
        return Err(ScheduleParseError::ExceedsDay);
    }
    let activities = minutes
        .map(ActivityMinutes::try_from)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| ScheduleParseError::ExceedsDay)?;
    Ok(DailySchedulePlan {
        activities,
        apprenticeship: OrganizationAllocation::parse(apprenticeship.0, apprenticeship.1)?,
        practice: OrganizationAllocation::parse(practice.0, practice.1)?,
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DailySchedule {
    /// Quiet study from a personally carried book or an on-site bookstore.
    pub reading_minutes: u16,
    /// Structured combat practice, including weapon drills, will, and balance.
    pub combat_training_minutes: u16,
    /// Social recreation which trains Charm at the activity rate.
    pub carousing_minutes: u16,
    /// Deliberate conversation with one selected relationship target. Its
    /// relationship resolution is strategic-module state, but it consumes
    /// discretionary time and therefore cannot also be restorative Leisure.
    pub socializing_minutes: u16,
    /// Supervised work in an unlocked profession.
    pub apprenticeship_minutes: u16,
    /// Independent paid professional work, available at Journeyman rank.
    pub profession_practice_minutes: u16,
    pub labor: u16,
    pub prayer: u16,
    pub thievery: u16,
    pub raiding: u16,
}

impl DailySchedule {
    pub fn allocated_minutes(&self) -> u64 {
        [
            self.labor,
            self.prayer,
            self.thievery,
            self.raiding,
            self.combat_training_minutes,
            self.carousing_minutes,
            self.socializing_minutes,
            self.apprenticeship_minutes,
            self.profession_practice_minutes,
            self.reading_minutes,
        ]
        .into_iter()
        .map(u64::from)
        .sum()
    }
}

/// Allocation validated independently of persistence and organization access.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidatedSchedule(DailySchedule);

impl TryFrom<DailySchedule> for ValidatedSchedule {
    type Error = ScheduleParseError;

    fn try_from(schedule: DailySchedule) -> Result<Self, Self::Error> {
        if schedule.allocated_minutes() > MINUTES_PER_DAY {
            return Err(ScheduleParseError::ExceedsDay);
        }
        for minutes in schedule
            .redistribution_minutes()
            .into_iter()
            .chain([schedule.reading_minutes])
        {
            ActivityMinutes::try_from(minutes)?;
        }
        Ok(Self(schedule))
    }
}

impl DailySchedule {
    // Authored activity order, shared by execution and both preview paths.
    // Reading remains unchanged: unavailable time does not add book study.
    fn redistribution_minutes(self) -> [u16; 9] {
        [
            self.combat_training_minutes,
            self.carousing_minutes,
            self.socializing_minutes,
            self.apprenticeship_minutes,
            self.profession_practice_minutes,
            self.labor,
            self.prayer,
            self.thievery,
            self.raiding,
        ]
    }
}

impl ValidatedSchedule {
    /// Calculate against the supplied context without consuming caller RNG or
    /// changing the saved allocation. Callers resolve current organization
    /// eligibility before constructing this value. The seed is character ID.
    pub fn effective_at(self, location: ActivityLocation, seed: u64) -> DailySchedule {
        let available = [
            true,
            location.allows(LocationActivity::Carousing),
            true,
            true,
            true,
            true,
            true,
            location.allows(LocationActivity::Thievery),
            location.allows(LocationActivity::Raiding),
        ];
        let planned = self.0.redistribution_minutes();
        let mut effective = planned;
        let mut segments = 0;
        for index in 0..planned.len() {
            if !available[index] {
                segments += planned[index] / ACTIVITY_SEGMENT_MINUTES;
                effective[index] = 0;
            }
        }
        let weights = std::array::from_fn::<_, 9, _>(|index| {
            if available[index] {
                u64::from(planned[index])
            } else {
                0
            }
        });
        if weights.iter().any(|weight| *weight != 0) {
            let mut random = StreamId::new("schedule.redistribution").rng(seed, &[]);
            for _ in 0..segments {
                let selected = random
                    .weighted_index(&weights)
                    .expect("validated daily minutes have a positive, bounded total");
                effective[selected] += ACTIVITY_SEGMENT_MINUTES;
            }
        }
        let [
            combat_training_minutes,
            carousing_minutes,
            socializing_minutes,
            apprenticeship_minutes,
            profession_practice_minutes,
            labor,
            prayer,
            thievery,
            raiding,
        ] = effective;
        DailySchedule {
            reading_minutes: self.0.reading_minutes,
            combat_training_minutes,
            carousing_minutes,
            socializing_minutes,
            apprenticeship_minutes,
            profession_practice_minutes,
            labor,
            prayer,
            thievery,
            raiding,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_replay_execution_with_socializing_and_unchanged_reading() {
        let proposed = DailySchedule {
            labor: 60,
            socializing_minutes: 120,
            raiding: 90,
            reading_minutes: 30,
            ..Default::default()
        };
        let validated = ValidatedSchedule::try_from(proposed).unwrap();
        let location = ActivityLocation::Settlement { has_inn: false };
        let mut labor = 0_u64;
        let mut social = 0_u64;
        for seed in 0..4_000 {
            let preview = validated.effective_at(location, seed);
            assert_eq!(preview, validated.effective_at(location, seed));
            assert_eq!(preview.allocated_minutes(), 300);
            assert_eq!(preview.reading_minutes, 30);
            assert_eq!(preview.raiding, 0);
            assert!(
                preview
                    .redistribution_minutes()
                    .iter()
                    .all(|minutes| minutes.is_multiple_of(ACTIVITY_SEGMENT_MINUTES))
            );
            labor += u64::from(preview.labor - proposed.labor);
            social += u64::from(preview.socializing_minutes - proposed.socializing_minutes);
        }
        assert!((1.9..2.1).contains(&(social as f64 / labor as f64)));
        assert_eq!(proposed.raiding, 90);
    }

    #[test]
    fn absent_alternatives_become_leisure_and_context_is_not_reserved() {
        let proposed = DailySchedule {
            raiding: 120,
            reading_minutes: 60,
            ..Default::default()
        };
        let validated = ValidatedSchedule::try_from(proposed).unwrap();
        let preview = validated.effective_at(ActivityLocation::NamedOutdoorLocation, 7);
        let execution = validated.effective_at(ActivityLocation::Settlement { has_inn: false }, 7);
        assert_eq!(preview, proposed);
        assert_eq!(
            execution,
            DailySchedule {
                reading_minutes: 60,
                ..Default::default()
            }
        );
    }

    #[test]
    fn invalid_allocations_fail_without_clamping() {
        assert_eq!(
            ValidatedSchedule::try_from(DailySchedule {
                labor: 1,
                ..Default::default()
            }),
            Err(ScheduleParseError::NotQuarterHour)
        );
        assert_eq!(
            ValidatedSchedule::try_from(DailySchedule {
                labor: 1440,
                reading_minutes: 15,
                ..Default::default()
            }),
            Err(ScheduleParseError::ExceedsDay)
        );
    }
}
