// Owns the persisted clock tables and stable schedule/activity wire model.
/// Natural recovery without useful medical support while taking full
/// settlement downtime.
pub const BASE_HEALTH_RECOVERED_PER_DAY: f32 = 0.01;
/// Additional daily recovery supplied by each point of the party Physiology
/// check. Checks are capped at the five-point scale used by the strategic UI.
pub const HEALTH_RECOVERED_PER_PHYSIOLOGY_CHECK_PER_DAY: f32 = 0.01;
pub const INN_GOLD_PER_DAY: u32 = adventuresim_core::strategic_economy::INN_FULL_BOARD_GOLD_PER_DAY;
const MIN_SETTLEMENT_REST_MINUTES: u64 = 60;
/// The current authoritative strategic time. `official_minutes` is absolute;
/// calendar presentation wraps it into years without making comparisons wrap.
/// The wall-clock epoch anchors calculation. Ordinary gameplay refreshes the
/// stored minute projection through `refresh_clock`. Capability-owned disposable
/// simulations also shift the epoch and stored minute together.
#[derive(Clone, Debug)]
#[table(accessor = world_clock, public)]
pub struct WorldClock {
    #[primary_key]
    pub id: u64,
    pub official_minutes: StrategicMinute,
    pub epoch_micros: i64,
}

/// Personal chronology, advanced by the actor's activity and family lifecycle.
/// Official world time initializes this clock; subsequent personal advancement
/// is a separate timeline rather than a second copy of the official minute.
#[derive(Clone, Debug)]
#[table(accessor = character_time)]
pub struct CharacterTime {
    #[primary_key]
    pub character_id: u64,
    /// Private full-table scan seam for trusted projections.
    #[index(btree)]
    pub scan_id: u64,
    pub minutes: StrategicMinute,
}

/// One 24-hour daily budget. Leisure is always the unallocated remainder.
#[derive(Clone, Debug, Default, SpacetimeType)]
pub struct ScheduleAllocation {
    pub reading_minutes: u16,
    pub combat_training_minutes: u16,
    pub carousing_minutes: u16,
    /// Allocated relationship time.  Unlike Carousing this neither requires
    /// an inn nor grants its incidental morale/incident outcome.
    pub socializing_minutes: u16,
    pub apprenticeship_minutes: u16,
    pub apprenticeship_organization_id: Option<String>,
    pub profession_practice_minutes: u16,
    pub practice_organization_id: Option<String>,
    /// Paid physical work; also trains Will at reduced speed.
    pub labor_minutes: u16,
    pub prayer_minutes: u16,
    pub thievery_minutes: u16,
    pub raiding_minutes: u16,
}

/// An explicit settlement activity selected by the player.  Profession
/// variants use the separate `service_id` reducer argument so this remains a
/// small, stable discriminator in generated clients.
#[derive(Clone, Copy, Debug, SpacetimeType)]
pub enum ImmediateActivity {
    Reading,
    Prayer,
    CombatTraining,
    Carousing,
    Apprenticeship,
    ProfessionPractice,
    Labor,
    Thievery,
    Raiding,
}

/// Daily settlement plan. Strategic travel never trains scheduled skills.
#[derive(Clone, Debug)]
#[table(accessor = character_training_schedule)]
pub struct CharacterTrainingSchedule {
    #[primary_key]
    pub character_id: u64,
    pub downtime: ScheduleAllocation,
}

impl ScheduleAllocation {
    pub fn allocated_minutes(&self) -> u64 {
        allocated_schedule_minutes([
            self.labor_minutes,
            self.prayer_minutes,
            self.thievery_minutes,
            self.raiding_minutes,
            self.combat_training_minutes,
            self.carousing_minutes,
            self.socializing_minutes,
            self.apprenticeship_minutes,
            self.profession_practice_minutes,
            self.reading_minutes,
        ])
    }
}
