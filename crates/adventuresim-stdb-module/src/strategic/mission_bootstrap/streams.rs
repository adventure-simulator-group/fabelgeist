//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const ACCEPTANCE: StreamId = StreamId::new("quest.acceptance");
pub(super) const ACCEPTANCE_CANDIDATE: StreamId = StreamId::new("quest.acceptance-candidate");
pub(super) const FIXTURE: StreamId = StreamId::new("quest.fixture");
pub(super) const FIXTURE_ATTEMPT: StreamId = StreamId::new("quest.fixture-attempt");
pub(super) const OBSERVER_HIGH: StreamId = StreamId::new("quest.observer-high");
pub(super) const OBSERVER_LOW: StreamId = StreamId::new("quest.observer-low");
pub(super) const SITE_BEARING: StreamId = StreamId::new("quest.site-bearing");
pub(super) const SITE_DISTANCE: StreamId = StreamId::new("quest.site-distance");
pub(super) const COMMAND: StreamId = StreamId::new("recruiting-party.command");
pub(super) const PHYSIOLOGY: StreamId = StreamId::new("recruiting-party.physiology");
pub(super) const RELIGION: StreamId = StreamId::new("recruiting-party.religion");
pub(super) const ROLE: StreamId = StreamId::new("recruiting-party.role");
pub(super) const QUEST_COUNT: StreamId = StreamId::new("settlement.quest-count");
pub(super) const RECRUITING_PARTY_COUNT: StreamId = StreamId::new("settlement.recruiting-party-count");

pub(super) fn recruiting_role(seed: u64) -> adventuresim_core::capability::RoleRequirements {
    let mut requirements = adventuresim_core::capability::RoleRequirements::default();
    let mut random = ROLE.rng(seed, &[]);
    if random.boolean() {
        requirements.melee = true;
    } else {
        requirements.ranged = true;
    }
    requirements.athletics = random.index(4) as u8;
    requirements.endurance = random.index(4) as u8;
    let armor = random.index(3);
    requirements.quarter_armor = armor == 1;
    requirements.half_armor = armor == 2;
    requirements.weapon_precision = random.index(4) as f32 * 0.5;
    requirements
}

pub(super) fn site_bearing(seed: u64, site_id: &str) -> f64 {
    fabelgeist_determinism::Seed::derive(
        &seed.to_le_bytes(),
        SITE_BEARING,
        &[site_id.as_bytes()],
    )
    .rng()
    .unit_f64()
        * std::f64::consts::TAU
}
